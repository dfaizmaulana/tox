/*! DisconnectNotification packet
*/

use toxcore::binary_io::*;
use nom::be_u8;

/** Sent by client to server.
Sent when client wants the server to forget about the connection related
to the connection_id in the notification. Server must remove this connection
and must be able to reuse the `connection_id` for another connection. If the
connection was connected the server must send a disconnect notification to the
other client. The other client must think that this client has simply
disconnected from the TCP server.

Sent by server to client.
Sent by the server to the client to tell them that the connection with
`connection_id` that was connected is now disconnected. It is sent either
when the other client of the connection disconnect or when they tell the
server to kill the connection (see above).

Serialized form:

Length | Content
------ | ------
`1`    | `0x03`
`1`    | connection_id

*/
#[derive(Debug, PartialEq, Clone)]
pub struct DisconnectNotification {
    /// The id of the disconnected client
    pub connection_id: u8
}

impl FromBytes for DisconnectNotification {
    named!(from_bytes<DisconnectNotification>, do_parse!(
        tag!("\x03") >>
        connection_id: be_u8 >>
        (DisconnectNotification { connection_id })
    ));
}

impl ToBytes for DisconnectNotification {
    fn to_bytes<'a>(&self, buf: (&'a mut [u8], usize)) -> Result<(&'a mut [u8], usize), GenError> {
        do_gen!(buf,
            gen_be_u8!(0x03) >>
            gen_be_u8!(self.connection_id)
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    encode_decode_test!(
        disconnect_notification_encode_decode,
        DisconnectNotification {
            connection_id: 17
        }
    );
}
