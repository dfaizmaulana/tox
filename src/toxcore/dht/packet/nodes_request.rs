/*
    Copyright (C) 2013 Tox project All Rights Reserved.
    Copyright © 2016-2017 Zetok Zalbavar <zexavexxe@gmail.com>
    Copyright © 2018 Namsoo CHO <nscho66@gmail.com>
    Copyright © 2018 Evgeny Kurnevsky <kurnevsky@gmail.com>
    Copyright © 2018 Roman Proskuryakov <humbug@deeptown.org>

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

/*! NodesRequest packet
*/

use nom::{be_u64, rest};

use std::io::{Error, ErrorKind};

use toxcore::binary_io::*;
use toxcore::crypto_core::*;
use toxcore::dht::codec::*;

/** Nodes request packet struct. It's used to get up to 4 closest nodes to
requested public key. Every 20 seconds DHT node sends `NodesRequest` packet to
a random node in kbucket and its known friends list.

https://zetok.github.io/tox-spec/#dht-packet

Length  | Content
------- | -------------------------
`1`     | `0x02`
`32`    | Public Key
`24`    | Nonce
`56`    | Payload

where Payload is encrypted [`NodesRequestPayload`](./struct.NodesRequestPayload.html)

*/
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodesRequest {
    /// public key used for payload encryption
    pub pk: PublicKey,
    /// one time serial number
    pub nonce: Nonce,
    /// encrypted payload
    pub payload: Vec<u8>,
}

impl ToBytes for NodesRequest {
    fn to_bytes<'a>(&self, buf: (&'a mut [u8], usize)) -> Result<(&'a mut [u8], usize), GenError> {
        do_gen!(buf,
            gen_be_u8!(0x02) >>
            gen_slice!(self.pk.as_ref()) >>
            gen_slice!(self.nonce.as_ref()) >>
            gen_slice!(self.payload.as_slice())
        )
    }
}

impl FromBytes for NodesRequest {
    named!(from_bytes<NodesRequest>, do_parse!(
        tag!("\x02") >>
        pk: call!(PublicKey::from_bytes) >>
        nonce: call!(Nonce::from_bytes) >>
        payload: map!(rest, |bytes| bytes.to_vec() ) >>
        (NodesRequest { pk, nonce, payload })
    ));
}

impl NodesRequest {
    /// create new NodesRequest object
    pub fn new(shared_secret: &PrecomputedKey, pk: &PublicKey, payload: NodesRequestPayload) -> NodesRequest {
        let nonce = gen_nonce();
        let mut buf = [0; MAX_DHT_PACKET_SIZE];
        let (_, size) = payload.to_bytes((&mut buf, 0)).unwrap();
        let payload = seal_precomputed(&buf[..size], &nonce, shared_secret);

        NodesRequest {
            pk: *pk,
            nonce,
            payload,
        }
    }
    /** Decrypt payload and try to parse it as `NodesRequestPayload`.

    Returns `Error` in case of failure:

    - fails to decrypt
    - fails to parse as given packet type
    */
    pub fn get_payload(&self, own_secret_key: &SecretKey) -> Result<NodesRequestPayload, Error> {
        debug!(target: "NodesRequest", "Getting packet data from NodesRequest.");
        trace!(target: "NodesRequest", "With NodesRequest: {:?}", self);
        let decrypted = open(&self.payload, &self.nonce, &self.pk, own_secret_key)
            .map_err(|()| {
                debug!("Decrypting NodesRequest failed!");
                Error::new(ErrorKind::Other, "NodesRequest decrypt error.")
            })?;

        match NodesRequestPayload::from_bytes(&decrypted) {
            IResult::Incomplete(e) => {
                debug!(target: "NodesRequest", "NodesRequestPayload deserialize error: {:?}", e);
                Err(Error::new(ErrorKind::Other,
                    format!("NodesRequestPayload deserialize error: {:?}", e)))
            },
            IResult::Error(e) => {
                debug!(target: "NodesRequest", "PingRequestPayload deserialize error: {:?}", e);
                Err(Error::new(ErrorKind::Other,
                    format!("NodesRequestPayload deserialize error: {:?}", e)))
            },
            IResult::Done(_, payload) => {
                Ok(payload)
            }
        }
    }
}

/** Request to get address of given DHT PK, or nodes that are closest in DHT
to the given PK. Request id is used for resistance against replay attacks.

Serialized form:

Length | Content
------ | ------
`32`   | DHT Public Key
`8`    | Request ID

Serialized form should be put in the encrypted part of `NodesRequest` packet.
*/
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct NodesRequestPayload {
    /// Public Key of the DHT node `NodesRequestPayload` is supposed to get address of.
    pub pk: PublicKey,
    /// An ID of the request.
    pub id: u64,
}

impl ToBytes for NodesRequestPayload {
    fn to_bytes<'a>(&self, buf: (&'a mut [u8], usize)) -> Result<(&'a mut [u8], usize), GenError> {
        do_gen!(buf,
            gen_slice!(self.pk.as_ref()) >>
            gen_be_u64!(self.id)
        )
    }
}

impl FromBytes for NodesRequestPayload {
    named!(from_bytes<NodesRequestPayload>, do_parse!(
        pk: call!(PublicKey::from_bytes) >>
        id: be_u64 >>
        eof!() >>
        (NodesRequestPayload { pk, id })
    ));
}

#[cfg(test)]
mod tests {
    use toxcore::dht::packet::nodes_request::*;
    use toxcore::dht::packet::DhtPacket;

    encode_decode_test!(
        nodes_request_payload_encode_decode,
        NodesRequestPayload { pk: gen_keypair().0, id: 42 }
    );

    dht_packet_encode_decode!(nodes_request_encode_decode, NodesRequest);

    dht_packet_encrypt_decrypt!(
        nodes_request_payload_encrypt_decrypt,
        NodesRequest,
        NodesRequestPayload { pk: gen_keypair().0, id: 42 }
    );

    dht_packet_encrypt_decrypt_invalid_key!(
        nodes_request_payload_encrypt_decrypt_invalid_key,
        NodesRequest,
        NodesRequestPayload { pk: gen_keypair().0, id: 42 }
    );

    dht_packet_decode_invalid!(nodes_request_decode_invalid, NodesRequest);
}
