use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use dashmap::DashMap;
use tokio::sync::mpsc;
use r_protocol::Frame;

pub type PeerId = [u8; 32];
pub type PeerMap = Arc<DashMap<PeerId, mpsc::Sender<Frame>>>;
pub type OfflineBuffer = Arc<DashMap<PeerId, VecDeque<(Instant, Frame)>>>;