//! # 网络框架（抽象层）
//!
//! 提供网络通信的抽象类型：可靠/不可靠通道、状态复制、预测回滚。
//!
//! **注意**: 本模块仅提供框架抽象，不包含实际的 socket I/O 或传输层实现。
//! 如需实际网络功能，请在游戏层集成 UDP/TCP 库（如 `laminar`、`quinn`）
//! 并通过本模块的类型进行数据编排。

use bevy_ecs::prelude::*;
use std::collections::VecDeque;

use crate::app::App;
use crate::plugin::Plugin;

// ---------------------------------------------------------------------------
//  Config & State
// ---------------------------------------------------------------------------

/// Network role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkRole {
    /// Authoritative server.
    Server,
    /// Client connected to a server.
    Client,
}

/// Network configuration resource.
#[derive(Debug, Clone, Resource)]
pub struct NetworkConfig {
    /// Server or client role.
    pub role: NetworkRole,
    /// Bind address (server listens, client sends from).
    pub bind_addr: String,
    /// Server address (for clients to connect to).
    pub server_addr: Option<String>,
    /// Server tick rate (ticks per second).
    pub tick_rate: u32,
    /// Maximum number of connected clients.
    pub max_clients: u32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            role: NetworkRole::Server,
            bind_addr: "0.0.0.0:7777".to_string(),
            server_addr: None,
            tick_rate: 20,
            max_clients: 32,
        }
    }
}

/// Network runtime state resource.
#[derive(Debug, Clone, Resource)]
pub struct NetworkState {
    /// Whether we are connected.
    pub connected: bool,
    /// Our assigned client ID (None if server or not yet connected).
    pub client_id: Option<u64>,
    /// Round-trip time in milliseconds.
    pub rtt_ms: f32,
    /// Total bytes sent.
    pub bytes_sent: u64,
    /// Total bytes received.
    pub bytes_received: u64,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            connected: false,
            client_id: None,
            rtt_ms: 0.0,
            bytes_sent: 0,
            bytes_received: 0,
        }
    }
}

// ---------------------------------------------------------------------------
//  Events
// ---------------------------------------------------------------------------

/// Network event.
///
/// 通过 `EventWriter<NetworkEvent>` 发送，`EventReader<NetworkEvent>` 接收。
#[derive(Debug, Clone, Event)]
pub enum NetworkEvent {
    /// A client connected.
    Connected {
        /// The client ID.
        client_id: u64,
    },
    /// A client disconnected.
    Disconnected {
        /// The client ID.
        client_id: u64,
    },
    /// Data received from a client.
    DataReceived {
        /// The client ID.
        client_id: u64,
        /// Channel number.
        channel: u8,
        /// Raw data payload.
        data: Vec<u8>,
    },
}

// ---------------------------------------------------------------------------
//  UDP Transport Abstraction
// ---------------------------------------------------------------------------

/// Packet header for reliable/unreliable channels.
#[derive(Debug, Clone, Copy)]
pub struct PacketHeader {
    /// Sequence number.
    pub sequence: u32,
    /// Last acknowledged sequence.
    pub ack: u32,
    /// Bitfield of previous 32 ACKs.
    pub ack_bitfield: u32,
    /// Channel ID.
    pub channel: u8,
}

/// A packet pending acknowledgement.
#[derive(Debug, Clone)]
pub struct PendingPacket {
    /// Sequence number.
    pub sequence: u32,
    /// Payload data.
    pub data: Vec<u8>,
    /// Time when the packet was sent (seconds since epoch or monotonic).
    pub send_time: f64,
    /// Number of retransmission attempts.
    pub retries: u32,
}

/// Reliable channel: tracks sent packets and processes ACKs.
#[derive(Debug)]
pub struct ReliableChannel {
    /// Next sequence number to assign.
    pub next_sequence: u32,
    /// Packets awaiting ACK.
    pub pending_acks: Vec<PendingPacket>,
}

impl ReliableChannel {
    /// Create a new reliable channel.
    pub fn new() -> Self {
        Self {
            next_sequence: 0,
            pending_acks: Vec::new(),
        }
    }

    /// Send data: assigns a sequence number, queues for ACK tracking, returns the header.
    pub fn send(&mut self, data: Vec<u8>, current_time: f64) -> PacketHeader {
        let seq = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        self.pending_acks.push(PendingPacket {
            sequence: seq,
            data,
            send_time: current_time,
            retries: 0,
        });
        PacketHeader {
            sequence: seq,
            ack: 0,
            ack_bitfield: 0,
            channel: 0,
        }
    }

    /// Process an ACK: remove the acknowledged packet and any covered by the bitfield.
    /// Uses wrapping arithmetic to handle sequence number wraparound.
    pub fn receive_ack(&mut self, ack: u32, ack_bitfield: u32) {
        self.pending_acks.retain(|p| {
            if p.sequence == ack {
                return false;
            }
            let diff = ack.wrapping_sub(p.sequence);
            if diff > 0 && diff <= 32 && (ack_bitfield & (1 << (diff - 1))) != 0 {
                return false;
            }
            true
        });
    }

    /// Get packets that need retransmission (send_time + timeout < current_time).
    pub fn get_resend_packets(&mut self, current_time: f64, timeout: f64) -> Vec<(u32, Vec<u8>)> {
        let mut result = Vec::new();
        for p in &mut self.pending_acks {
            if current_time - p.send_time > timeout {
                result.push((p.sequence, p.data.clone()));
                p.send_time = current_time;
                p.retries += 1;
            }
        }
        result
    }
}

impl Default for ReliableChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Unreliable channel: assigns sequence numbers without ACK tracking.
#[derive(Debug)]
pub struct UnreliableChannel {
    /// Next sequence number.
    pub next_sequence: u32,
}

impl UnreliableChannel {
    /// Create a new unreliable channel.
    pub fn new() -> Self {
        Self { next_sequence: 0 }
    }

    /// Send data: assigns sequence, no ACK tracking.
    pub fn send(&mut self, _data: &[u8]) -> PacketHeader {
        let seq = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        PacketHeader {
            sequence: seq,
            ack: 0,
            ack_bitfield: 0,
            channel: 1,
        }
    }
}

impl Default for UnreliableChannel {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
//  ECS Replication
// ---------------------------------------------------------------------------

/// Marker component for entities that should be replicated over the network.
#[derive(Debug, Clone, Copy, Component)]
pub struct Replicated;

/// Unique network entity identifier.
#[derive(Debug, Clone, Copy, Component)]
pub struct NetworkId {
    /// The unique network ID.
    pub id: u64,
}

/// Buffer of entity state changes to be sent over the network.
#[derive(Resource, Default)]
pub struct ReplicationBuffer {
    /// Changed entities: (network_id, serialized delta bytes).
    pub changed_entities: Vec<(u64, Vec<u8>)>,
}

impl ReplicationBuffer {
    /// Push a change for a network entity.
    pub fn push_change(&mut self, id: u64, data: Vec<u8>) {
        self.changed_entities.push((id, data));
    }
    /// Drain all changes.
    pub fn drain(&mut self) -> Vec<(u64, Vec<u8>)> {
        std::mem::take(&mut self.changed_entities)
    }
}

/// Simple XOR-based delta encoder/decoder.
pub struct DeltaEncoder;

impl DeltaEncoder {
    /// Encode a delta: XOR old and new byte slices.
    pub fn encode(old: &[u8], new: &[u8]) -> Vec<u8> {
        let len = old.len().max(new.len());
        let mut delta = Vec::with_capacity(len);
        for i in 0..len {
            let a = old.get(i).copied().unwrap_or(0);
            let b = new.get(i).copied().unwrap_or(0);
            delta.push(a ^ b);
        }
        delta
    }

    /// Decode a delta: XOR base and delta byte slices.
    pub fn decode(base: &[u8], delta: &[u8]) -> Vec<u8> {
        let len = base.len().max(delta.len());
        let mut result = Vec::with_capacity(len);
        for i in 0..len {
            let a = base.get(i).copied().unwrap_or(0);
            let d = delta.get(i).copied().unwrap_or(0);
            result.push(a ^ d);
        }
        result
    }
}

// ---------------------------------------------------------------------------
//  Client Prediction & Rollback
// ---------------------------------------------------------------------------

/// State history for client-side prediction and rollback.
#[derive(Debug, Clone)]
pub struct PredictionState<T> {
    /// History of (tick, state) pairs.
    pub history: VecDeque<(u32, T)>,
    /// Maximum number of history entries.
    pub max_history: usize,
}

impl<T> PredictionState<T> {
    /// Create a new prediction state buffer.
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::new(),
            max_history,
        }
    }

    /// Push a state snapshot for the given tick.
    pub fn push(&mut self, tick: u32, state: T) {
        self.history.push_back((tick, state));
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    /// Get the state at the given tick.
    pub fn get(&self, tick: u32) -> Option<&T> {
        self.history.iter().find(|(t, _)| *t == tick).map(|(_, s)| s)
    }

    /// Remove all states after the given tick (for rollback).
    pub fn rollback_to(&mut self, tick: u32) {
        while let Some((t, _)) = self.history.back() {
            if *t > tick {
                self.history.pop_back();
            } else {
                break;
            }
        }
    }
}

impl<T> Default for PredictionState<T> {
    fn default() -> Self {
        Self::new(128)
    }
}

/// Input buffer resource for client-side prediction.
#[derive(Resource)]
pub struct InputBuffer {
    /// Buffered inputs: (tick, serialized input bytes).
    pub inputs: VecDeque<(u32, Vec<u8>)>,
    /// Current simulation tick.
    pub current_tick: u32,
    /// Maximum number of buffered inputs before eviction.
    pub max_size: usize,
}

impl InputBuffer {
    /// Push an input for the given tick. Evicts oldest entries if buffer exceeds max_size.
    pub fn push_input(&mut self, tick: u32, input: Vec<u8>) {
        self.inputs.push_back((tick, input));
        while self.inputs.len() > self.max_size {
            self.inputs.pop_front();
        }
    }

    /// Get the input at the given tick.
    pub fn get_input(&self, tick: u32) -> Option<&[u8]> {
        self.inputs.iter().find(|(t, _)| *t == tick).map(|(_, d)| d.as_slice())
    }
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self {
            inputs: VecDeque::new(),
            current_tick: 0,
            max_size: 1024,
        }
    }
}

// ---------------------------------------------------------------------------
//  Systems
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
//  Plugin
// ---------------------------------------------------------------------------

/// Network plugin. Initializes network resources and registers cleanup systems.
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkConfig>();
        app.init_resource::<NetworkState>();
        app.add_event::<NetworkEvent>();
        app.init_resource::<ReplicationBuffer>();
        app.init_resource::<InputBuffer>();
    }

    fn name(&self) -> &str {
        "NetworkPlugin"
    }
}

// ---------------------------------------------------------------------------
//  Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reliable_channel_send_ack() {
        let mut ch = ReliableChannel::new();
        let h1 = ch.send(vec![1, 2, 3], 0.0);
        assert_eq!(h1.sequence, 0);
        let h2 = ch.send(vec![4, 5, 6], 0.1);
        assert_eq!(h2.sequence, 1);
        assert_eq!(ch.pending_acks.len(), 2);

        // ACK packet 0
        ch.receive_ack(0, 0);
        assert_eq!(ch.pending_acks.len(), 1);
        assert_eq!(ch.pending_acks[0].sequence, 1);

        // ACK packet 1
        ch.receive_ack(1, 0);
        assert!(ch.pending_acks.is_empty());
    }

    #[test]
    fn test_reliable_channel_resend() {
        let mut ch = ReliableChannel::new();
        ch.send(vec![1, 2], 0.0);
        ch.send(vec![3, 4], 0.0);

        // No resend within timeout
        let resend = ch.get_resend_packets(0.5, 1.0);
        assert!(resend.is_empty());

        // After timeout
        let resend = ch.get_resend_packets(1.5, 1.0);
        assert_eq!(resend.len(), 2);
    }

    #[test]
    fn test_delta_encoder() {
        let old = vec![10, 20, 30, 40];
        let new = vec![10, 25, 30, 50];
        let delta = DeltaEncoder::encode(&old, &new);
        let decoded = DeltaEncoder::decode(&old, &delta);
        assert_eq!(decoded, new);
    }

    #[test]
    fn test_delta_encoder_different_lengths() {
        let old = vec![1, 2, 3];
        let new = vec![1, 2, 3, 4, 5];
        let delta = DeltaEncoder::encode(&old, &new);
        let decoded = DeltaEncoder::decode(&old, &delta);
        assert_eq!(decoded, new);
    }

    #[test]
    fn test_prediction_state() {
        let mut ps: PredictionState<Vec3> = PredictionState::new(10);
        ps.push(0, Vec3::ZERO);
        ps.push(1, Vec3::X);
        ps.push(2, Vec3::Y);

        assert_eq!(ps.get(1), Some(&Vec3::X));
        assert_eq!(ps.get(5), None);

        ps.rollback_to(1);
        assert_eq!(ps.history.len(), 2);
        assert_eq!(ps.get(2), None);
    }

    #[test]
    fn test_input_buffer() {
        let mut buf = InputBuffer::default();
        buf.push_input(0, vec![1, 0, 0]);
        buf.push_input(1, vec![0, 1, 0]);

        assert_eq!(buf.get_input(0), Some(&[1, 0, 0][..]));
        assert_eq!(buf.get_input(1), Some(&[0, 1, 0][..]));
        assert_eq!(buf.get_input(2), None);
    }

    #[test]
    fn test_network_plugin() {
        let mut app = crate::app::App::new();
        let plugin = NetworkPlugin;
        plugin.build(&mut app);

        assert!(app.world.get_resource::<NetworkConfig>().is_some());
        assert!(app.world.get_resource::<NetworkState>().is_some());
        // NetworkEvent 现在通过 Events<NetworkEvent> 注册
        assert!(app.world.get_resource::<Events<NetworkEvent>>().is_some());
    }

    use glam::Vec3;
}
