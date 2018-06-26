/*! OnionDataResponse packet
*/

use super::*;

use toxcore::binary_io::*;
use toxcore::crypto_core::*;

use nom::rest;

/** When onion node receives `OnionDataRequest` packet it converts it to
`OnionDataResponse` and sends to destination node if it announced itself
and is contained in onion nodes list.

Serialized form:

Length   | Content
-------- | ------
`1`      | `0x86`
`24`     | `Nonce`
`32`     | Temporary `PublicKey`
variable | Payload

*/
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnionDataResponse {
    /// Nonce for the current encrypted payload
    pub nonce: Nonce,
    /// Temporary `PublicKey` for the current encrypted payload
    pub temporary_pk: PublicKey,
    /// Encrypted payload
    pub payload: Vec<u8>
}

impl FromBytes for OnionDataResponse {
    named!(from_bytes<OnionDataResponse>, do_parse!(
        verify!(rest_len, |len| len <= ONION_MAX_PACKET_SIZE) >>
        tag!(&[0x86][..]) >>
        nonce: call!(Nonce::from_bytes) >>
        temporary_pk: call!(PublicKey::from_bytes) >>
        payload: rest >>
        (OnionDataResponse {
            nonce,
            temporary_pk,
            payload: payload.to_vec()
        })
    ));
}

impl ToBytes for OnionDataResponse {
    fn to_bytes<'a>(&self, buf: (&'a mut [u8], usize)) -> Result<(&'a mut [u8], usize), GenError> {
        do_gen!(buf,
            gen_be_u8!(0x86) >>
            gen_slice!(self.nonce.as_ref()) >>
            gen_slice!(self.temporary_pk.as_ref()) >>
            gen_slice!(self.payload) >>
            gen_len_limit(ONION_MAX_PACKET_SIZE)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    encode_decode_test!(
        onion_data_response_encode_decode,
        OnionDataResponse {
            nonce: gen_nonce(),
            temporary_pk: gen_keypair().0,
            payload: vec![42; 123]
        }
    );
}
