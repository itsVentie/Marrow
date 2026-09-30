use dashmap::DashMap;
use r_protocol::Frame;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

pub type PeerId = [u8; 32];
pub type PeerMap = Arc<DashMap<PeerId, mpsc::Sender<Frame>>>;
pub type OfflineBuffer = Arc<DashMap<PeerId, VecDeque<(Instant, Frame)>>>;
