extern crate tox;
extern crate futures;
extern crate tokio;
extern crate tokio_codec;

#[macro_use]
extern crate log;
extern crate env_logger;

use tox::toxcore::crypto_core::*;
use tox::toxcore::tcp::handshake::make_server_handshake;
use tox::toxcore::tcp::codec;
use tox::toxcore::tcp::server::{Server, ServerProcessor};

use futures::prelude::*;

use tokio::util::FutureExt;
use tokio::net::TcpListener;
use tokio_codec::Framed;

use std::time;
use std::io::{Error, ErrorKind};

fn main() {
    env_logger::init();
    // Server constant PK for examples/tests
    // Use `gen_keypair` to generate random keys
    let server_pk = PublicKey([177, 185, 54, 250, 10, 168, 174,
                            148, 0, 93, 99, 13, 131, 131, 239,
                            193, 129, 141, 80, 158, 50, 133, 100,
                            182, 179, 183, 234, 116, 142, 102, 53, 38]);
    let server_sk = SecretKey([74, 163, 57, 111, 32, 145, 19, 40,
                            44, 145, 233, 210, 173, 67, 88, 217,
                            140, 147, 14, 176, 106, 255, 54, 249,
                            159, 12, 18, 39, 123, 29, 125, 230]);
    let addr = "0.0.0.0:12345".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    info!("Listening on addr={}, {:?}", addr, &server_pk);

    // Ignore all TCP onion requests for now
    let server_inner = Server::new();

    // TODO move this processing future into a standalone library function
    let server = listener.incoming().for_each(move |socket| {
        let addr = socket.peer_addr()
            .map_err(|e| {
                error!("could not get peer addr: {}", e);
                e
            })?;
        debug!("A new client connected from {}", addr);

        let register_client = make_server_handshake(socket, server_sk.clone())
            .map_err(|e|
                Error::new(ErrorKind::Other,
                    format!("Handshake error {:?}", e))
            )
            .map(|(socket, channel, client_pk)| {
                debug!("Handshake for client {:?} complited", &client_pk);
                (socket, channel, client_pk)
            });

        let server_inner_c = server_inner.clone();
        let process = register_client.and_then(move |(socket, channel, client_pk)| {
            let secure_socket = Framed::new(socket, codec::Codec::new(channel));
            let (to_client, from_client) = secure_socket.split();
            let ServerProcessor { from_client_tx, to_client_rx, processor } =
                ServerProcessor::create(
                    server_inner_c,
                    client_pk.clone(),
                    addr.ip(),
                    addr.port()
                );

            // writer = for each Packet from to_client_rx send it to client
            let writer = to_client_rx
                .map_err(|()| Error::from(ErrorKind::UnexpectedEof))
                .fold(to_client, move |to_client, packet| {
                    debug!("Send {:?} => {:?}", client_pk, packet);
                    to_client.send(packet)
                        .deadline(time::Instant::now() + time::Duration::from_secs(30))
                        .map_err(|_|
                            Error::new(ErrorKind::Other,
                                format!("Writer timed out"))
                        )
                })
                // drop to_client when to_client_rx stream is exhausted
                .map(|_to_client| ())
                .map_err(|_|
                    Error::new(ErrorKind::Other,
                        format!("Writer ended with error"))
                );

            // reader = for each Packet from client send it to server processor
            let reader = from_client
                .forward(from_client_tx
                    .sink_map_err(|e|
                        Error::new(ErrorKind::Other,
                            format!("Could not forward message from client to server {:?}", e))
                    )
                )
                .map(|(_from_client, _sink_err)| ())
                .map_err(|_|
                    Error::new(ErrorKind::Other,
                        format!("Reader ended with error"))
                );

            processor
                .select(reader)
                    .map_err(move |(err, _select_next)| {
                        err
                    })
                    .map(|_| ())
                .select(writer)
                    .map_err(move |(err, _select_next)| {
                        err
                    })
        });

        tokio::spawn( process.map(|_| ()).map_err(|_| ()) );

        Ok(())
    })
    .map_err(|err| {
            // All tasks must have an `Error` type of `()`. This forces error
            // handling and helps avoid silencing failures.
            //
            // In our example, we are only going to log the error to STDOUT.
            println!("listener.incoming() error = {:?}", err);
    });
    tokio::run(server);
}
