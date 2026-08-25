use crate::behaviour::{MarrowBehaviour, MarrowBehaviourEvent};
use crate::codec::{MarrowProtocol, MarrowRequest, MarrowResponse};
use libp2p::{
    autonat, identify, identity,
    kad::{store::MemoryStore, Behaviour as Kademlia, Config as KademliaConfig},
    ping,
    request_response::{Behaviour as RequestResponse, Config as ReqRespConfig, ProtocolSupport},
    swarm::SwarmEvent,
    Multiaddr, PeerId, StreamProtocol, Swarm,
};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum NetworkCommand {
    StartListening {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send + Sync>>>,
    },
    Dial {
        peer_id: PeerId,
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send + Sync>>>,
    },
    SendFrame {
        peer_id: PeerId,
        data: Vec<u8>,
        sender: oneshot::Sender<Result<Vec<u8>, Box<dyn Error + Send + Sync>>>,
    },
}

#[derive(Debug)]
pub enum NetworkEvent {
    FrameReceived {
        peer_id: PeerId,
        data: Vec<u8>,
    },
}

pub struct NetworkNode {
    swarm: Swarm<MarrowBehaviour>,
    command_receiver: mpsc::Receiver<NetworkCommand>,
    event_sender: mpsc::Sender<NetworkEvent>,
    pending_responses: HashMap<
        libp2p::request_response::OutboundRequestId,
        oneshot::Sender<Result<Vec<u8>, Box<dyn Error + Send + Sync>>>,
    >,
}

impl NetworkNode {
    pub fn new(
        keypair: identity::Keypair,
    ) -> Result<(Self, mpsc::Sender<NetworkCommand>, mpsc::Receiver<NetworkEvent>), Box<dyn Error>>
    {
        let local_peer_id = PeerId::from(keypair.public());

        let proto = StreamProtocol::new("/marrow/kad/1.0.0");
        let kad_config = KademliaConfig::new(proto);
        let store = MemoryStore::new(local_peer_id);
        let kademlia = Kademlia::with_config(local_peer_id, store, kad_config);

        let identify = identify::Behaviour::new(identify::Config::new(
            "/marrow/1.0.0".to_string(),
            keypair.public(),
        ));

        let ping = ping::Behaviour::new(ping::Config::default());
        let autonat = autonat::Behaviour::new(local_peer_id, autonat::Config::default());

        let req_resp = RequestResponse::new(
            [(MarrowProtocol, ProtocolSupport::Full)],
            ReqRespConfig::default(),
        );

        let behaviour = MarrowBehaviour {
            kademlia,
            identify,
            ping,
            autonat,
            req_resp,
        };

        let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
            .with_tokio()
            .with_tcp(
                libp2p::tcp::Config::default(),
                libp2p::noise::Config::new,
                libp2p::yamux::Config::default,
            )?
            .with_quic()
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        let (cmd_tx, cmd_rx) = mpsc::channel(32);
        let (event_tx, event_rx) = mpsc::channel(32);

        let node = Self {
            swarm,
            command_receiver: cmd_rx,
            event_sender: event_tx,
            pending_responses: HashMap::new(),
        };

        Ok((node, cmd_tx, event_rx))
    }

    pub async fn run(mut self) {
        use futures::StreamExt;
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_swarm_event(event).await,
                command = self.command_receiver.recv() => {
                    match command {
                        Some(cmd) => self.handle_command(cmd).await,
                        None => break,
                    }
                }
            }
        }
    }

    async fn handle_command(&mut self, command: NetworkCommand) {
        match command {
            NetworkCommand::StartListening { addr, sender } => {
                let res = self
                    .swarm
                    .listen_on(addr)
                    .map(|_| ())
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>);
                let _ = sender.send(res);
            }
            NetworkCommand::Dial { peer_id, addr, sender } => {
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer_id, addr.clone());
                let res = self
                    .swarm
                    .dial(addr)
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>);
                let _ = sender.send(res);
            }
            NetworkCommand::SendFrame { peer_id, data, sender } => {
                let req_id = self
                    .swarm
                    .behaviour_mut()
                    .req_resp
                    .send_request(&peer_id, MarrowRequest(data));
                self.pending_responses.insert(req_id, sender);
            }
        }
    }

    async fn handle_swarm_event(&mut self, event: SwarmEvent<MarrowBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(MarrowBehaviourEvent::ReqResp(
                libp2p::request_response::Event::Message { peer, message },
            )) => match message {
                libp2p::request_response::Message::Request {
                    request_id: _,
                    request,
                    channel,
                } => {
                    let _ = self
                        .event_sender
                        .send(NetworkEvent::FrameReceived {
                            peer_id: peer,
                            data: request.0,
                        })
                        .await;
                    let _ = self
                        .swarm
                        .behaviour_mut()
                        .req_resp
                        .send_response(channel, MarrowResponse(vec![1]));
                }
                libp2p::request_response::Message::Response { request_id, response } => {
                    if let Some(sender) = self.pending_responses.remove(&request_id) {
                        let _ = sender.send(Ok(response.0));
                    }
                }
            },
            _ => {}
        }
    }
}
