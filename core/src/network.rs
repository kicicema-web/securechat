use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use libp2p::{
    gossipsub::{self, IdentTopic, MessageAuthenticity},
    identity::Keypair,
    noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    PeerId, SwarmBuilder,
};
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::time::Duration;

use crate::protocol::ProtocolMessage;

/// Network event types
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// New message received
    MessageReceived {
        peer_id: String,
        message: ProtocolMessage,
    },
    /// Peer discovered
    PeerDiscovered {
        peer_id: String,
        addrs: Vec<String>,
    },
    /// Peer connected
    PeerConnected {
        peer_id: String,
    },
    /// Peer disconnected
    PeerDisconnected {
        peer_id: String,
    },
    /// Connection established
    Connected,
    /// Connection lost
    Disconnected,
}

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub listen_addrs: Vec<String>,
    pub bootstrap_peers: Vec<String>,
    pub enable_mdns: bool,
    pub topic: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addrs: vec![
                "/ip4/0.0.0.0/tcp/0".to_string(),
                "/ip4/0.0.0.0/udp/0/quic-v1".to_string(),
            ],
            bootstrap_peers: vec![],
            enable_mdns: true,
            topic: "securechat-v1".to_string(),
        }
    }
}

/// Network behaviour combining all protocols
#[derive(NetworkBehaviour)]
struct SecureChatBehaviour {
    gossipsub: gossipsub::Behaviour,
}

/// P2P Network manager
pub struct NetworkManager {
    local_peer_id: PeerId,
    event_sender: mpsc::Sender<NetworkEvent>,
    command_receiver: mpsc::Receiver<NetworkCommand>,
    config: NetworkConfig,
}

/// Commands that can be sent to the network manager
#[derive(Debug)]
pub enum NetworkCommand {
    SendMessage {
        peer_id: Option<String>, // None = broadcast
        message: ProtocolMessage,
    },
    ConnectPeer {
        addr: String,
    },
    DisconnectPeer {
        peer_id: String,
    },
    Shutdown,
}

impl NetworkManager {
    /// Create new network manager
    pub fn new(
        config: NetworkConfig,
    ) -> Result<(Self, mpsc::Receiver<NetworkEvent>, mpsc::Sender<NetworkCommand>)> {
        let (event_sender, event_receiver) = mpsc::channel(100);
        let (command_sender, command_receiver) = mpsc::channel(100);
        
        // Generate deterministic keypair from identity
        // In real app, load from secure storage
        let local_key = Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        
        log::info!("Local peer ID: {}", local_peer_id);
        
        let manager = Self {
            local_peer_id,
            event_sender,
            command_receiver,
            config,
        };
        
        Ok((manager, event_receiver, command_sender))
    }
    
    /// Start the network event loop
    pub async fn run(mut self) -> Result<()> {
        // Generate keypair for swarm
        let local_key = Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        
        // Build swarm using new libp2p 0.54+ API
        let mut swarm = SwarmBuilder::with_existing_identity(local_key)
            .with_async_std()
            .with_tcp(
                libp2p::tcp::Config::default(),
                noise::Config::new,
                libp2p::yamux::Config::default,
            )?
            .with_quic()
            .with_behaviour(|keypair| {
                // Gossipsub configuration
                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .heartbeat_interval(Duration::from_secs(10))
                    .validation_mode(gossipsub::ValidationMode::Strict)
                    .mesh_outbound_min(4)
                    .mesh_n_low(4)
                    .mesh_n(6)
                    .mesh_n_high(12)
                    .gossip_lazy(6)
                    .history_length(10)
                    .history_gossip(3)
                    .build()
                    .expect("Valid gossipsub config");
                
                let gossipsub = gossipsub::Behaviour::new(
                    MessageAuthenticity::Signed(keypair.clone()),
                    gossipsub_config,
                ).expect("Valid gossipsub behaviour");
                
                SecureChatBehaviour {
                    gossipsub,
                }
            })?
            .build();
        
        // Subscribe to topic
        let topic = IdentTopic::new(&self.config.topic);
        swarm.behaviour_mut().gossipsub.subscribe(&topic)
            .context("Failed to subscribe to topic")?;
        
        // Listen on addresses
        for addr in &self.config.listen_addrs {
            swarm.listen_on(addr.parse()?)
                .context("Failed to listen on address")?;
        }
        
        // Dial bootstrap peers
        for addr in &self.config.bootstrap_peers {
            let multiaddr: libp2p::Multiaddr = addr.parse()?;
            swarm.dial(multiaddr)
                .context("Failed to dial bootstrap peer")?;
        }
        
        log::info!("Network started");
        
        // Event loop
        loop {
            futures::select! {
                event = swarm.select_next_some() => {
                    self.handle_swarm_event(&mut swarm, event, &topic).await?;
                }
                command = self.command_receiver.next() => {
                    if let Some(cmd) = command {
                        if self.handle_command(&mut swarm, cmd, &topic).await? {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        
        log::info!("Network stopped");
        Ok(())
    }
    
    async fn handle_swarm_event(
        &mut self,
        swarm: &mut libp2p::Swarm<SecureChatBehaviour>,
        event: SwarmEvent<SecureChatBehaviourEvent>,
        topic: &IdentTopic,
    ) -> Result<()> {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                log::info!("Listening on {}", address);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                log::info!("Connected to {}", peer_id);
                self.event_sender.send(NetworkEvent::PeerConnected {
                    peer_id: peer_id.to_string(),
                }).await.ok();
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                log::info!("Disconnected from {}", peer_id);
                self.event_sender.send(NetworkEvent::PeerDisconnected {
                    peer_id: peer_id.to_string(),
                }).await.ok();
            }
            SwarmEvent::Behaviour(SecureChatBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source,
                message_id: _,
                message,
            })) => {
                match bincode::deserialize::<ProtocolMessage>(&message.data) {
                    Ok(protocol_msg) => {
                        self.event_sender.send(NetworkEvent::MessageReceived {
                            peer_id: propagation_source.to_string(),
                            message: protocol_msg,
                        }).await.ok();
                    }
                    Err(e) => {
                        log::warn!("Failed to deserialize message: {}", e);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    async fn handle_command(
        &mut self,
        swarm: &mut libp2p::Swarm<SecureChatBehaviour>,
        command: NetworkCommand,
        topic: &IdentTopic,
    ) -> Result<bool> {
        match command {
            NetworkCommand::SendMessage { peer_id, message } => {
                let data = bincode::serialize(&message)
                    .context("Failed to serialize message")?;
                
                if let Some(_target) = peer_id {
                    // Direct message (requires established connection)
                    swarm.behaviour_mut().gossipsub.publish(
                        topic.clone(),
                        data,
                    ).ok();
                } else {
                    // Broadcast
                    swarm.behaviour_mut().gossipsub.publish(
                        topic.clone(),
                        data,
                    ).ok();
                }
            }
            NetworkCommand::ConnectPeer { addr } => {
                let multiaddr: libp2p::Multiaddr = addr.parse()?;
                swarm.dial(multiaddr)
                    .context("Failed to dial peer")?;
            }
            NetworkCommand::DisconnectPeer { peer_id } => {
                if let Ok(pid) = peer_id.parse::<PeerId>() {
                    swarm.disconnect_peer_id(pid).ok();
                }
            }
            NetworkCommand::Shutdown => {
                return Ok(true);
            }
        }
        Ok(false)
    }
    
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }
}

/// Peer connection manager for direct connections
pub struct PeerManager {
    known_peers: HashMap<String, PeerInfo>,
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub peer_id: String,
    pub public_key: [u8; 32],
    pub display_name: Option<String>,
    pub last_seen: std::time::Instant,
    pub addresses: Vec<String>,
    pub trusted: bool,
}

impl PeerManager {
    pub fn new() -> Self {
        Self {
            known_peers: HashMap::new(),
        }
    }
    
    pub fn add_peer(&mut self, info: PeerInfo) {
        self.known_peers.insert(info.peer_id.clone(), info);
    }
    
    pub fn get_peer(&self, peer_id: &str) -> Option<&PeerInfo> {
        self.known_peers.get(peer_id)
    }
    
    pub fn update_last_seen(&mut self, peer_id: &str) {
        if let Some(peer) = self.known_peers.get_mut(peer_id) {
            peer.last_seen = std::time::Instant::now();
        }
    }
    
    pub fn get_trusted_peers(&self) -> Vec<&PeerInfo> {
        self.known_peers.values().filter(|p| p.trusted).collect()
    }
}

/// Utility functions for network operations
pub mod utils {
    use super::*;
    
    /// Parse a multiaddress string
    pub fn parse_multiaddr(addr: &str) -> Result<libp2p::Multiaddr> {
        addr.parse()
            .context("Invalid multiaddress")
    }
    
    /// Generate QR code data for sharing contact
    pub fn generate_contact_qr(public_key: &[u8; 32], display_name: &str) -> String {
        use base64::Engine;
        format!("securechat://contact?key={}&name={}",
            base64::engine::general_purpose::STANDARD.encode(public_key),
            display_name
        )
    }
    
    /// Parse contact from QR code
    pub fn parse_contact_qr(_qr: &str) -> Result<(String, [u8; 32])> {
        // Implementation would parse the QR data
        // For now, return error
        Err(anyhow::anyhow!("QR parsing not implemented"))
    }
}
