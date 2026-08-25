use crate::codec::MarrowCodec;
use libp2p::autonat;
use libp2p::dcutr;
use libp2p::identify;
use libp2p::kad::{store::MemoryStore, Behaviour as Kademlia};
use libp2p::ping;
use libp2p::relay;
use libp2p::request_response::Behaviour as RequestResponse;
use libp2p::swarm::NetworkBehaviour;

#[derive(NetworkBehaviour)]
pub struct MarrowBehaviour {
    pub kademlia: Kademlia<MemoryStore>,
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
    pub autonat: autonat::Behaviour,
    pub relay_client: relay::client::Behaviour,
    pub dcutr: dcutr::Behaviour,
    pub req_resp: RequestResponse<MarrowCodec>,
}
