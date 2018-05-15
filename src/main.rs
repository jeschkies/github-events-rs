extern crate tokio;

use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;
use std::str;
use std::net::Shutdown;

/// Incredible simple hello world HTTP server.
/// Simply call with `curl localhost:6142`.
fn main() {
    let addr = "127.0.0.1:6142".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let server = listener.incoming().for_each(|socket| {
        println!("accepted socket; addr={:?}", socket.peer_addr().unwrap());

        let buf = Vec::with_capacity(256);
        let connection = io::read_until(socket, '\n' as u8, buf)
            .and_then(|(socket, data)| {
                let d: Vec<u8> = data;
                //let s = str::from_utf8(&d).unwrap();
                println!("Received {:?}", d);
                io::write_all(socket, "HTTP/1.1 200 OK\n\n Hello World").then(|res|{
                    match res {
                        Ok((socket, _)) => {
                            println!("wrote message; success=true");
                            socket.shutdown(Shutdown::Both).unwrap();
                        },
                        Err(_) => {
                            println!("wrote message; success=true");
                        },
                    };
                    Ok(())
                })
            })
            .map_err(|err| {
                eprintln!("IO error {:?}", err)
            });

        tokio::spawn(connection);
        Ok(())
    })
    .map_err(|err| {
        println!("accept error = {:?}", err);
    });

    println!("server running on localhost:6142");
    tokio::run(server);
}
