/*! Top-level DHT udp packets according
    to [Tox spec](https://zetok.github.io/tox-spec/#packet-kind)
*/

#[cfg(test)]
#[macro_use]
mod macros;
mod ping_request;
mod ping_response;
mod nodes_request;
mod nodes_response;
mod dht_request;
mod cookie_request;
mod cookie_response;
mod bootstrap_info;
mod lan_discovery;
mod crypto_handshake;
mod crypto_data;
mod cookie;

pub use self::ping_request::*;
pub use self::ping_response::*;
pub use self::nodes_request::*;
pub use self::nodes_response::*;
pub use self::dht_request::*;
pub use self::cookie_request::*;
pub use self::cookie_response::*;
pub use self::bootstrap_info::*;
pub use self::lan_discovery::*;
pub use self::crypto_handshake::*;
pub use self::crypto_data::*;
pub use self::cookie::*;

use toxcore::binary_io::*;
use toxcore::onion::packet::*;

/** DHT packet enum that encapsulates all types of DHT packets.
*/
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DhtPacket {
    /// [`PingRequest`](./struct.PingRequest.html) structure.
    PingRequest(PingRequest),
    /// [`PingResponse`](./struct.PingResponse.html) structure.
    PingResponse(PingResponse),
    /// [`NodesRequest`](./struct.NodesRequest.html) structure.
    NodesRequest(NodesRequest),
    /// [`NodesResponse`](./struct.NodesResponse.html) structure.
    NodesResponse(NodesResponse),
    /// [`CookieRequest`](./struct.CookieRequest.html) structure.
    CookieRequest(CookieRequest),
    /// [`CookieRequest`](./struct.CookieRequest.html) structure.
    CookieResponse(CookieResponse),
    /// [`CryptoHandshake`](./struct.CryptoHandshake.html) structure.
    CryptoHandshake(CryptoHandshake),
    /// [`CryptoData`](./struct.CryptoData.html) structure.
    CryptoData(CryptoData),
    /// [`DhtRequest`](./struct.DhtRequest.html) structure.
    DhtRequest(DhtRequest),
    /// [`LanDiscovery`](./struct.LanDiscovery.html) structure.
    LanDiscovery(LanDiscovery),
    /// [`OnionRequest0`](../onion/struct.OnionRequest0.html) structure.
    OnionRequest0(OnionRequest0),
    /// [`OnionRequest1`](../onion/struct.OnionRequest1.html) structure.
    OnionRequest1(OnionRequest1),
    /// [`OnionRequest2`](../onion/struct.OnionRequest2.html) structure.
    OnionRequest2(OnionRequest2),
    /// [`OnionAnnounceRequest`](../onion/struct.OnionAnnounceRequest.html) structure.
    OnionAnnounceRequest(OnionAnnounceRequest),
    /// [`OnionAnnounceResponse`](../onion/struct.OnionAnnounceResponse.html) structure.
    OnionAnnounceResponse(OnionAnnounceResponse),
    /// [`OnionDataRequest`](../onion/struct.OnionDataRequest.html) structure.
    OnionDataRequest(OnionDataRequest),
    /// [`OnionDataResponse`](../onion/struct.OnionDataResponse.html) structure.
    OnionDataResponse(OnionDataResponse),
    /// [`OnionResponse3`](../onion/struct.OnionResponse3.html) structure.
    OnionResponse3(OnionResponse3),
    /// [`OnionResponse2`](../onion/struct.OnionResponse2.html) structure.
    OnionResponse2(OnionResponse2),
    /// [`OnionResponse1`](../onion/struct.OnionResponse1.html) structure.
    OnionResponse1(OnionResponse1),
    /// [`BootstrapInfo`](./struct.BootstrapInfo.html) structure.
    BootstrapInfo(BootstrapInfo)
}

impl ToBytes for DhtPacket {
    fn to_bytes<'a>(&self, buf: (&'a mut [u8], usize)) -> Result<(&'a mut [u8], usize), GenError> {
        match *self {
            DhtPacket::PingRequest(ref p) => p.to_bytes(buf),
            DhtPacket::PingResponse(ref p) => p.to_bytes(buf),
            DhtPacket::NodesRequest(ref p) => p.to_bytes(buf),
            DhtPacket::NodesResponse(ref p) => p.to_bytes(buf),
            DhtPacket::CookieRequest(ref p) => p.to_bytes(buf),
            DhtPacket::CookieResponse(ref p) => p.to_bytes(buf),
            DhtPacket::CryptoHandshake(ref p) => p.to_bytes(buf),
            DhtPacket::CryptoData(ref p) => p.to_bytes(buf),
            DhtPacket::DhtRequest(ref p) => p.to_bytes(buf),
            DhtPacket::LanDiscovery(ref p) => p.to_bytes(buf),
            DhtPacket::OnionRequest0(ref p) => p.to_bytes(buf),
            DhtPacket::OnionRequest1(ref p) => p.to_bytes(buf),
            DhtPacket::OnionRequest2(ref p) => p.to_bytes(buf),
            DhtPacket::OnionAnnounceRequest(ref p) => p.to_bytes(buf),
            DhtPacket::OnionAnnounceResponse(ref p) => p.to_bytes(buf),
            DhtPacket::OnionDataRequest(ref p) => p.to_bytes(buf),
            DhtPacket::OnionDataResponse(ref p) => p.to_bytes(buf),
            DhtPacket::OnionResponse3(ref p) => p.to_bytes(buf),
            DhtPacket::OnionResponse2(ref p) => p.to_bytes(buf),
            DhtPacket::OnionResponse1(ref p) => p.to_bytes(buf),
            DhtPacket::BootstrapInfo(ref p) => p.to_bytes(buf)
        }
    }
}

impl FromBytes for DhtPacket {
    named!(from_bytes<DhtPacket>, alt!(
        map!(PingRequest::from_bytes, DhtPacket::PingRequest) |
        map!(PingResponse::from_bytes, DhtPacket::PingResponse) |
        map!(NodesRequest::from_bytes, DhtPacket::NodesRequest) |
        map!(NodesResponse::from_bytes, DhtPacket::NodesResponse) |
        map!(CookieRequest::from_bytes, DhtPacket::CookieRequest) |
        map!(CookieResponse::from_bytes, DhtPacket::CookieResponse) |
        map!(CryptoHandshake::from_bytes, DhtPacket::CryptoHandshake) |
        map!(CryptoData::from_bytes, DhtPacket::CryptoData) |
        map!(DhtRequest::from_bytes, DhtPacket::DhtRequest) |
        map!(LanDiscovery::from_bytes, DhtPacket::LanDiscovery) |
        map!(OnionRequest0::from_bytes, DhtPacket::OnionRequest0) |
        map!(OnionRequest1::from_bytes, DhtPacket::OnionRequest1) |
        map!(OnionRequest2::from_bytes, DhtPacket::OnionRequest2) |
        map!(OnionAnnounceRequest::from_bytes, DhtPacket::OnionAnnounceRequest) |
        map!(OnionAnnounceResponse::from_bytes, DhtPacket::OnionAnnounceResponse) |
        map!(OnionDataRequest::from_bytes, DhtPacket::OnionDataRequest) |
        map!(OnionDataResponse::from_bytes, DhtPacket::OnionDataResponse) |
        map!(OnionResponse3::from_bytes, DhtPacket::OnionResponse3) |
        map!(OnionResponse2::from_bytes, DhtPacket::OnionResponse2) |
        map!(OnionResponse1::from_bytes, DhtPacket::OnionResponse1) |
        map!(BootstrapInfo::from_bytes, DhtPacket::BootstrapInfo)
    ));
}
