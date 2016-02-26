/*
    Copyright (C) 2013 Tox project All Rights Reserved.
    Copyright © 2016 Zetok Zalbavar <zexavexxe@gmail.com>

    This file is part of Tox.

    Tox is libre software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Tox is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Tox.  If not, see <http://www.gnu.org/licenses/>.
*/


// ↓ FIXME doc
//! DHT part of the toxcore.

use ip::*;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use toxcore::binary_io::*;
use toxcore::crypto_core::*;


/// Type of [`Ping`](./struct.Ping.html) packet. Either a request or response.
///
/// * `0` – if ping is a request;
/// * `1` – if ping is a response.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PingType {
    /// Request ping response.
    Req  = 0,
    /// Respond to ping request.
    Resp = 1,
}

/// Uses the first byte from the provided slice to de-serialize
/// [`PingType`](./enum.PingType.html). Returns `None` if first byte of slice
/// doesn't match `PingType` or slice has no elements.
impl FromBytes<PingType> for PingType {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() == 0 { return None }
        match bytes[0] {
            0 => Some(PingType::Req),
            1 => Some(PingType::Resp),
            _ => None,
        }
    }
}


/// Used to request/respond to ping. Use in an encrypted form in DHT packets.
///
/// Serialized form:
///
/// ```text
///                 (9 bytes)
/// +-------------------------+
/// | Ping type     (1 byte ) |
/// | ping_id       (8 bytes) |
/// +-------------------------+
/// ```
///
/// Serialized form should be put in the encrypted part of DHT packet.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Ping {
    p_type: PingType,
    /// An ID of the request. Response ID must match ID of the request,
    /// otherwise ping is invalid.
    pub id: u64,
}

/// Length in bytes of [`Ping`](./struct.Ping.html) when serialized into bytes.
pub const PING_SIZE: usize = 9;

impl Ping {
    /// Create new ping request with a randomly generated `id`.
    pub fn new() -> Self {
        Ping { p_type: PingType::Req, id: random_u64(), }
    }

    /// Check whether given `Ping` is a request.
    pub fn is_request(&self) -> bool {
        self.p_type == PingType::Req
    }

    /// Create answer to ping request. Returns `None` if supplied `Ping` is
    /// already a ping response.
    // TODO: make sure that checking whether `Ping` is not a response is needed
    //       here
    pub fn response(&self) -> Option<Self> {
        if self.p_type == PingType::Resp {
            return None;
        }

        Some(Ping { p_type: PingType::Resp, id: self.id })
    }
}

/// Serializes [`Ping`](./struct.Ping.html) into bytes.
impl AsBytes for Ping {
    fn as_bytes(&self) -> Vec<u8> {
        let mut res = Vec::with_capacity(PING_SIZE);
        // `PingType`
        res.push(self.p_type as u8);
        // And random ping_id as bytes
        res.extend_from_slice(&u64_to_array(self.id));
        res
    }
}

/// De-seralize [`Ping`](./struct.Ping.html) from bytes. Tries to parse first
/// [`PING_SIZE`](./constant.PING_SIZE.html) bytes from supplied slice as
/// `Ping`.
impl FromBytes<Ping> for Ping {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < PING_SIZE { return None; }
        if let Some(ping_type) = PingType::from_bytes(bytes) {
            return Some(Ping {
                p_type: ping_type,
                id: array_to_u64(&[bytes[1], bytes[2], bytes[3], bytes[4],
                                   bytes[5], bytes[6], bytes[7], bytes[8]]),
            })
        }
        None  // parsing failed
    }
}


/// Used by [`PackedNode`](./struct.PackedNode.html).
///
/// * 1st bit – protocol
/// * 3 bits – `0`
/// * 4th bit – address family
///
/// Values:
///
/// * `2` – UDP IPv4
/// * `10` – UDP IPv6
/// * `130` – TCP IPv4
/// * `138` – TCP IPv6
///
/// DHT module *should* use only UDP variants of `IpType`, given that DHT runs
/// solely over the UDP.
///
/// TCP variants are to be used for sending/receiving info about TCP relays.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IpType {
    /// UDP over IPv4.
    U4 = 2,
    /// UDP over IPv6.
    U6 = 10,
    /// TCP over IPv4.
    T4 = 130,
    /// TCP over IPv6.
    T6 = 138,
}

/// Match first byte from the provided slice as `IpType`. If no match found,
/// return `None`.
impl FromBytes<IpType> for IpType {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() == 0 { return None }
        match bytes[0] {
            2   => Some(IpType::U4),
            10  => Some(IpType::U6),
            130 => Some(IpType::T4),
            138 => Some(IpType::T6),
            _   => None,
        }
    }
}


// TODO: move it somewhere else
impl AsBytes for IpAddr {
    fn as_bytes(&self) -> Vec<u8> {
        match *self {
            IpAddr::V4(a) => a.octets().iter().map(|b| *b).collect(),
            IpAddr::V6(a) => {
                let mut result: Vec<u8> = vec![];
                for n in a.segments().iter() {
                    result.extend_from_slice(&u16_to_array(*n));
                }
                result
            },
        }
    }
}


// TODO: move it somewhere else
/// Can fail if there are less than 16 bytes supplied, otherwise parses first
/// 16 bytes as an `Ipv6Addr`.
impl FromBytes<Ipv6Addr> for Ipv6Addr {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 16 { return None }

        let (a, b, c, d, e, f, g, h) = {
            let mut v: Vec<u16> = Vec::with_capacity(8);
            for slice in bytes[..16].chunks(2) {
                v.push(array_to_u16(&[slice[0], slice[1]]));
            }
            (v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7])
        };
        Some(Ipv6Addr::new(a, b, c, d, e, f, g, h))
    }
}


// TODO: probably needs to be renamed & moved out of DHT, given that it most
// likely will be used not only for DHT node info, but also for TCP relay info.
/// `Packed Node` format is a way to store the node info in a small yet easy to
/// parse format.
///
/// It is used in many places in Tox, e.g. in `DHT Send nodes`.
///
/// To store more than one node, simply append another on to the previous one:
///
/// `[packed node 1][packed node 2][...]`
///
/// Packed node format:
///
/// ```text
///                          (39 bytes for IPv4, 51 for IPv6)
/// +-----------------------------------+
/// | ip_type                ( 1 byte ) |
/// |                                   |
/// | IPv4 Address           ( 4 bytes) |
/// |  -OR-                     -OR-    |
/// | IPv6 Address           (16 bytes) |
/// | Port                   ( 2 bytes) |
/// | Node ID                (32 bytes) |
/// +-----------------------------------+
/// ```
///
/// DHT module *should* use only UDP variants of `IpType`, given that DHT runs
/// solely on the UDP.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PackedNode {
    /// IP type, includes also info about protocol used.
    pub ip_type: IpType,
    socketaddr: SocketAddr,
    node_id: PublicKey,
}

/// Size in bytes of serialized [`PackedNode`](./struct.PackedNode.html) with
/// IPv4.
pub const PACKED_NODE_IPV4_SIZE: usize = PUBLICKEYBYTES + 7;
/// Size in bytes of serialized [`PackedNode`](./struct.PackedNode.html) with
/// IPv6.
pub const PACKED_NODE_IPV6_SIZE: usize = PUBLICKEYBYTES + 19;

impl PackedNode {
    /// New `PackedNode`.
    //
    // TODO: Should fail if type of IP address supplied in
    // `socketaddr` doesn't match `IpType`..?
    pub fn new(ip_type: IpType,
               socketaddr: SocketAddr,
               node_id: &PublicKey) -> Self {
        PackedNode {
            ip_type: ip_type,
            socketaddr: socketaddr,
            node_id: *node_id,
        }
    }

    /// Get an IP address from the `PackedNode`.
    pub fn ip(&self) -> IpAddr {
        match self.socketaddr {
            SocketAddr::V4(addr) => IpAddr::V4(*addr.ip()),
            SocketAddr::V6(addr) => IpAddr::V6(*addr.ip()),
        }
    }

    /// Parse bytes into multiple `PackedNode`s.
    ///
    /// If provided bytes are smaller than [`PACKED_NODE_IPV4_SIZE`]
    /// (./constant.PACKED_NODE_IPV4_SIZE.html) or can't be parsed, returns
    /// `None`.
    ///
    /// Parses nodes until first error is encountered.
    pub fn from_bytes_multiple(bytes: &[u8]) -> Option<Vec<PackedNode>> {
        if bytes.len() < PACKED_NODE_IPV4_SIZE { return None }

        let mut cur_pos = 0;
        let mut result = vec![];

        while let Some(node) = PackedNode::from_bytes(&bytes[cur_pos..]) {
            cur_pos += {
                match node.ip_type {
                    IpType::U4 | IpType::T4 => PACKED_NODE_IPV4_SIZE,
                    IpType::U6 | IpType::T6 => PACKED_NODE_IPV6_SIZE,
                }
            };
            result.push(node);
        }

        if result.len() == 0 {
            return None
        } else {
            return Some(result)
        }
    }

}

/// Serialize `PackedNode` into bytes.
///
/// Can be either [`PACKED_NODE_IPV4_SIZE`]
/// (./constant.PACKED_NODE_IPV4_SIZE.html) or [`PACKED_NODE_IPV6_SIZE`]
/// (./constant.PACKED_NODE_IPV6_SIZE.html) bytes long, depending on whether
/// IPv4 or IPv6 is being used.
impl AsBytes for PackedNode {
    fn as_bytes(&self) -> Vec<u8> {
        // TODO: ↓ perhaps capacity PACKED_NODE_IPV6_SIZE ?
        let mut result: Vec<u8> = Vec::with_capacity(PACKED_NODE_IPV4_SIZE);

        result.push(self.ip_type as u8);

        let addr: Vec<u8> = self.ip().as_bytes();
        result.extend_from_slice(&addr);
        // port
        result.extend_from_slice(&u16_to_array(self.socketaddr.port()));

        let PublicKey(ref pk) = self.node_id;
        result.extend_from_slice(pk);

        result
    }
}

/// Deserialize bytes into `PackedNode`. Returns `None` if deseralizing
/// failed.
///
/// Can fail if:
///
///  - length is too short for given [`IpType`](./enum.IpType.html)
///  - PK can't be parsed
///
/// Blindly trusts that provided `IpType` matches - i.e. if there are provided
/// 51 bytes (which is lenght of `PackedNode` that contains IPv6), and `IpType`
/// says that it's actually IPv4, bytes will be parsed as if that was an IPv4
/// address.
impl FromBytes<PackedNode> for PackedNode {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        // parse bytes as IPv4
        fn as_ipv4(bytes: &[u8]) -> Option<(SocketAddr, PublicKey)> {
            let addr = Ipv4Addr::new(bytes[1], bytes[2], bytes[3], bytes[4]);
            let port = array_to_u16(&[bytes[5], bytes[6]]);
            let saddr = SocketAddrV4::new(addr, port);

            let pk = match PublicKey::from_slice(&bytes[7..PACKED_NODE_IPV4_SIZE]) {
                Some(pk) => pk,
                None => return None,
            };

            Some((SocketAddr::V4(saddr), pk))
        }

        // parse bytes as IPv4
        fn as_ipv6(bytes: &[u8]) -> Option<(SocketAddr, PublicKey)> {
            if bytes.len() < PACKED_NODE_IPV6_SIZE { return None }

            let addr = match Ipv6Addr::from_bytes(&bytes[1..]) {
                Some(a) => a,
                None    => return None,
            };
            let port = array_to_u16(&[bytes[17], bytes[18]]);
            let saddr = SocketAddrV6::new(addr, port, 0, 0);

            let pk = match PublicKey::from_slice(&bytes[19..PACKED_NODE_IPV6_SIZE]) {
                Some(p) => p,
                None    => return None,
            };

            Some((SocketAddr::V6(saddr), pk))
        }


        if bytes.len() >= PACKED_NODE_IPV4_SIZE {
            let (iptype, saddr_and_pk) = match IpType::from_bytes(bytes) {
                Some(IpType::U4) => (IpType::U4, as_ipv4(bytes)),
                Some(IpType::T4) => (IpType::T4, as_ipv4(bytes)),
                Some(IpType::U6) => (IpType::U6, as_ipv6(bytes)),
                Some(IpType::T6) => (IpType::T6, as_ipv6(bytes)),
                None => return None,
            };

            let (saddr, pk) = match saddr_and_pk {
                Some(v) => v,
                None => return None,
            };

            return Some(PackedNode {
                ip_type: iptype,
                socketaddr: saddr,
                node_id: pk,
            });
        }
        // `if` not triggered, make sure to return `None`
        None
    }
}


// TODO: make sure ↓ it's correct
/// Request to get address of given DHT PK, or nodes that are closest in DHT
/// to the given PK.
///
/// Packet type `2`.
///
/// Serialized form:
///
/// ```text
/// +-----------------------------------+
/// | DHT PUBKEY             (32 bytes) |
/// | ping_id                ( 8 bytes) |
/// +-----------------------------------+
/// ```
///
/// Serialized form should be put in the encrypted part of DHT packet.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct GetNodes {
    /// Public Key of the DHT node `GetNodes` is supposed to get address of.
    pub pk: PublicKey,
    /// An ID of the request.
    pub id: u64,
}

/// Size of serialized [`GetNodes`](./struct.GetNodes.html) in bytes.
pub const GET_NODES_SIZE: usize = PUBLICKEYBYTES + 8;

impl GetNodes {
    /// Create new `GetNodes` with given PK.
    pub fn new(their_public_key: &PublicKey) -> Self {
        GetNodes { pk: *their_public_key, id: random_u64() }
    }
}

/// Serialization of `GetNodes`. Resulting lenght should be
/// [`GET_NODES_SIZE`](./constant.GET_NODES_SIZE.html).
impl AsBytes for GetNodes {
    fn as_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(GET_NODES_SIZE);
        let PublicKey(pk_bytes) = self.pk;
        result.extend_from_slice(&pk_bytes);
        result.extend_from_slice(&u64_to_array(self.id));
        result
    }
}

/// De-serialization of bytes into `GetNodes`. If less than
/// [`GET_NODES_SIZE`](./constant.GET_NODES_SIZE.html) bytes are provided,
/// de-serialization will fail, returning `None`.
impl FromBytes<GetNodes> for GetNodes {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < GET_NODES_SIZE { return None }
        if let Some(pk) = PublicKey::from_slice(&bytes[..PUBLICKEYBYTES]) {
            // need shorter name for ID bytes
            let b = &bytes[PUBLICKEYBYTES..GET_NODES_SIZE];
            let id = array_to_u64(&[b[0], b[1], b[2], b[3],
                                    b[4], b[5], b[6], b[7]]);
            return Some(GetNodes { pk: pk, id: id })
        }
        None  // de-serialization failed
    }
}


/// Response to [`GetNodes`](./struct.GetNodes.html) request, containing up to
/// `4` nodes closest to the requested node.
///
/// Packet type `4`.
///
/// Serialized form:
///
/// ```text
/// +-----------------------------------+
/// | Encrypted payload:                |
/// | Number of packed nodes ( 1 byte ) |
/// | Nodes in packed format (max of 4) |
/// |      (39 bytes for IPv4) * number |
/// |      (51 bytes for IPv6) * number |
/// | ping_id                ( 8 bytes) |
/// +-----------------------------------+
/// ```
///
/// Serialized form should be put in the encrypted part of DHT packet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SendNodes {
    /// Nodes sent in response to [`GetNodes`](./struct.GetNodes.html) request.
    ///
    /// There can be only 1 to 4 nodes in `SendNodes`.
    pub nodes: Vec<PackedNode>,
    /// Ping id that was received in [`GetNodes`](./struct.GetNodes.html)
    /// request.
    pub id: u64,
}

impl SendNodes {
    /// Create new `SendNodes`. Returns `None` if 0 or more than 4 nodes are
    /// supplied.
    ///
    /// Created as an answer to `GetNodes` request.
    pub fn from_request(request: &GetNodes, nodes: Vec<PackedNode>) -> Option<Self> {
        if nodes.len() == 0 || nodes.len() > 4 { return None }

        Some(SendNodes { nodes: nodes, id: request.id })
    }
}

/// Method assumes that supplied `SendNodes` has correct number of nodes
/// included – `[1, 4]`.
impl AsBytes for SendNodes {
    fn as_bytes(&self) -> Vec<u8> {
        // first byte is number of nodes
        let mut result: Vec<u8> = vec![self.nodes.len() as u8];
        for node in &*self.nodes {
            result.extend_from_slice(&node.as_bytes());
        }
        result.extend_from_slice(&u64_to_array(self.id));
        result
    }
}

/// Method to parse received bytes as `SendNodes`.
///
/// Returns `None` if bytes can't be parsed into `SendNodes`.
impl FromBytes<SendNodes> for SendNodes {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        // first byte should say how many `PackedNode`s `SendNodes` has.
        // There has to be at least 1 node, and no more than 4.
        if bytes[0] < 1 || bytes[0] > 4 { return None }

        if let Some(nodes) = PackedNode::from_bytes_multiple(&bytes[1..]) {
            if nodes.len() > 4 { return None }

            // since 1st byte is a number of nodes
            let mut nodes_bytes_len = 1;
            // TODO: ↓ most likely can be done more efficiently
            for node in &nodes {
                nodes_bytes_len += node.as_bytes().len();
            }

            // need u64 from bytes
            let mut ping_id: [u8; 8] = [0; 8];
            for pos in 0..ping_id.len() {
                ping_id[pos] = bytes[nodes_bytes_len + pos];
            }

            return Some(SendNodes { nodes: nodes, id: array_to_u64(&ping_id) })
        }
        None  // parsing failed
    }
}

/// Types of DHT packets that can be put in `DHT Packet`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DPacketT {
    /// `Ping` packet type.
    Ping(Ping),
    /// `GetNodes` packet type. Used to request nodes.
    // TODO: rename to `GetN()` ? – consistency with DPacketTnum
    GetNodes(GetNodes),
    /// `SendNodes` response to `GetNodes` request.
    // TODO: rename to `SendN()` ? – consistency with DPacketTnum
    SendNodes(SendNodes),
}

impl DPacketT {
    /// Provide packet type number.
    ///
    /// To use for serialization: `.as_type() as u8`.
    pub fn as_type(&self) -> DPacketTnum {
        match *self {
            DPacketT::GetNodes(_) => DPacketTnum::GetN,
            DPacketT::SendNodes(_) => DPacketTnum::SendN,
            DPacketT::Ping(p) => {
                if p.is_request() {
                    DPacketTnum::PingReq
                } else {
                    DPacketTnum::PingResp
                }
            },
        }
    }
}

impl AsBytes for DPacketT {
    fn as_bytes(&self) -> Vec<u8> {
        match *self {
            DPacketT::Ping(ref d)      => d.as_bytes(),
            DPacketT::GetNodes(ref d)  => d.as_bytes(),
            DPacketT::SendNodes(ref d) => d.as_bytes(),
        }
    }
}


/// Packet type number associated with packet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DPacketTnum {
    /// [`Ping`](./struct.Ping.html) request number.
    PingReq  = 0,
    /// [`Ping`](./struct.Ping.html) response number.
    PingResp = 1,
    /// [`GetNodes`](./struct.GetNodes.html) packet number.
    GetN     = 2,
    /// [`SendNodes`](./struct.SendNodes.html) packet number.
    SendN    = 4,
}

/// Parse first byte from provided `bytes` as `DPacketTnum`.
///
/// Returns `None` if no bytes provided, or first byte doesn't match.
impl FromBytes<DPacketTnum> for DPacketTnum {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() == 0 { return None }

        match bytes[0] {
            0 => Some(DPacketTnum::PingReq),
            1 => Some(DPacketTnum::PingResp),
            2 => Some(DPacketTnum::GetN),
            4 => Some(DPacketTnum::SendN),
            _ => None,
        }
    }
}


/// Standard DHT packet that encapsulates in the encrypted payload
/// [`DhtPacketT`](./enum.DhtPacketT.html).
///
/// Length      | Contents
/// ----------- | --------
/// `1`         | `uint8_t` packet kind (see other packets)
/// `32`        | Sender DHT Public Key
/// `24`        | Random nonce
/// variable    | Encrypted payload
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DhtPacket {
    packet_type: DPacketTnum,
    /// Public key of sender.
    pub sender_pk: PublicKey,
    nonce: Nonce,
    payload: Vec<u8>,
}

// TODO: max dht packet size?
/// Minimal size of [`DhtPacket`](./struct.DhtPacket.html) in bytes.
pub const DHT_PACKET_MIN_SIZE: usize = 1 // packet type, plain
                                     + PUBLICKEYBYTES
                                     + NONCEBYTES
                                     + MACBYTES
                                     + PING_SIZE; // smallest payload

// TODO: perhaps methods `is_ping(&self)` `is_get(&self)`, `is_send(&self)`
impl DhtPacket {
    /// Create new `DhtPacket`.
    // TODO: perhaps switch to using precomputed symmetric key?
    //        - given that computing shared key is apparently the most
    //          costly operation when it comes to crypto, using precomputed
    //          key might (would significantly?) lower resource usage
    pub fn new(own_secret_key: &SecretKey, own_public_key: &PublicKey,
               receiver_public_key: &PublicKey, nonce: &Nonce, packet: DPacketT)
        -> Self {

        let payload = seal(&packet.as_bytes(), nonce, receiver_public_key,
                           own_secret_key);

        DhtPacket {
            packet_type: packet.as_type(),
            sender_pk: *own_public_key,
            nonce: *nonce,
            payload: payload,
        }
    }

    /// Get packet data. This functino decrypts payload and tries to parse it
    /// as packet type.
    ///
    /// Returns `None` in case of faliure.
    pub fn get_packet(&self, own_secret_key: &SecretKey) -> Option<DPacketT> {
        let decrypted = match open(&self.payload, &self.nonce, &self.sender_pk,
                            own_secret_key) {
            Ok(d) => d,
            Err(_) => return None,
        };

        match self.packet_type {
            DPacketTnum::PingReq | DPacketTnum::PingResp => {
                if let Some(p) = Ping::from_bytes(&decrypted) {
                    return Some(DPacketT::Ping(p))
                }
            },
            DPacketTnum::GetN => {
                if let Some(n) = GetNodes::from_bytes(&decrypted) {
                    return Some(DPacketT::GetNodes(n))
                }
            },
            DPacketTnum::SendN => {
                if let Some(n) = SendNodes::from_bytes(&decrypted) {
                    return Some(DPacketT::SendNodes(n))
                }
            },
        }
        None  // parsing failed
    }
}

/// Serialize `DhtPacket` into bytes.
impl AsBytes for DhtPacket {
    fn as_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(DHT_PACKET_MIN_SIZE);
        result.push(self.packet_type as u8);

        let PublicKey(pk) = self.sender_pk;
        result.extend_from_slice(&pk);

        let Nonce(nonce) = self.nonce;
        result.extend_from_slice(&nonce);

        result.extend_from_slice(&self.payload);
        result
    }
}

/// De-serialize bytes into `DhtPacket`.
impl FromBytes<DhtPacket> for DhtPacket {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < DHT_PACKET_MIN_SIZE { return None }

        let packet_type = match DPacketTnum::from_bytes(bytes) {
            Some(b) => b,
            None => return None,
        };

        const NONCE_POS: usize = 1 + PUBLICKEYBYTES;
        let sender_pk = match PublicKey::from_slice(&bytes[1..NONCE_POS]) {
            Some(pk) => pk,
            None => return None,
        };

        const PAYLOAD_POS: usize = NONCE_POS + NONCEBYTES;
        let nonce = match Nonce::from_slice(&bytes[NONCE_POS..PAYLOAD_POS]) {
            Some(n) => n,
            None => return None,
        };

        Some(DhtPacket {
            packet_type: packet_type,
            sender_pk: sender_pk,
            nonce: nonce,
            payload: bytes[(PAYLOAD_POS)..].to_vec(),
        })
    }
}
