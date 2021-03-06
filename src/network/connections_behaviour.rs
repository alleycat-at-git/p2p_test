//! Contains network behavior for establishing connections with newly discovered peers

use libp2p::{
    core::connection::{ConnectedPoint, ConnectionId},
    swarm::protocols_handler::DummyProtocolsHandler,
    swarm::DialPeerCondition,
    swarm::{NetworkBehaviour, NetworkBehaviourAction, PollParameters, ProtocolsHandler},
    Multiaddr, PeerId,
};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

/// Network behaviour for adding new connections
pub struct ConnectionsBehaviour {
    new_peers: Arc<Mutex<Vec<PeerId>>>,
    peers: HashSet<PeerId>,
}

impl ConnectionsBehaviour {
    /// Create new behaviour
    pub fn new() -> Self {
        Self {
            peers: HashSet::new(),
            new_peers: Arc::new(Mutex::new(vec![])),
        }
    }

    /// Add peer to connect. The connection will be established
    /// if added peer id is greater than local id (to avoid duplex connections)
    pub fn insert_peer(&mut self, peer: PeerId) {
        if self.peers.contains(&peer) {
            return;
        }
        self.peers.insert(peer.clone());
        if let Ok(mut new_peers_ref) = self.new_peers.lock() {
            new_peers_ref.push(peer);
        } else {
            log::error!("Poisoned mutex in ConnectionsBehaviour");
        }
    }
}

impl NetworkBehaviour for ConnectionsBehaviour {
    type ProtocolsHandler = DummyProtocolsHandler;
    type OutEvent = ();

    fn new_handler(&mut self) -> Self::ProtocolsHandler {
        DummyProtocolsHandler::default()
    }

    fn addresses_of_peer(&mut self, _: &PeerId) -> Vec<Multiaddr> {
        Vec::new()
    }

    fn inject_connected(&mut self, _: &PeerId) {}

    fn inject_connection_established(&mut self, _: &PeerId, _: &ConnectionId, _: &ConnectedPoint) {}

    fn inject_disconnected(&mut self, _: &PeerId) {}

    fn inject_connection_closed(&mut self, _: &PeerId, _: &ConnectionId, _: &ConnectedPoint) {}

    fn inject_event(
        &mut self,
        _: PeerId,
        _: ConnectionId,
        _: <Self::ProtocolsHandler as ProtocolsHandler>::OutEvent,
    ) {
    }

    fn poll(
        &mut self,
        _: &mut Context<'_>,
        params: &mut impl PollParameters,
    ) -> Poll<
        NetworkBehaviourAction<
            <Self::ProtocolsHandler as ProtocolsHandler>::InEvent,
            Self::OutEvent,
        >,
    > {
        if let Ok(mut new_peers_ref) = self.new_peers.lock() {
            if let Some(peer_id) = new_peers_ref.pop() {
                let self_peer_id = params.local_peer_id();
                // Avoid duplex connections
                if self_peer_id < &peer_id {
                    log::debug!("Connecting peer {:?}", peer_id);
                    return Poll::Ready(NetworkBehaviourAction::DialPeer {
                        peer_id,
                        condition: DialPeerCondition::Disconnected,
                    });
                } else {
                    log::debug!("Waiting for connection from peer {:?}", peer_id);
                    return Poll::Pending;
                }
            }
        } else {
            log::error!("Poisoned mutex in ConnectionsBehaviour");
        }
        Poll::Pending
    }
}
