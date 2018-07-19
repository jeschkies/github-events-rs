extern crate hyper;
extern crate futures;
extern crate futures_cpupool;

use futures::Stream;
use futures::future::Future;
use futures::sink::Sink;
use futures::sync::mpsc::{channel, Receiver, Sender};
use futures_cpupool::CpuPool;

use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};

use std::{thread, time};

struct HelloWorld {
    sender: Sender<String>,
}

impl HelloWorld {
    fn new(tx: Sender<String>) -> HelloWorld {
        HelloWorld { sender: tx }
    }
}

const PHRASE: &'static str = "Hello, World!";

impl Service for HelloWorld {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;

    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        // TODO: parse body.
        let path = String::from(req.uri().path());
        let tx = self.sender.clone();
        Box::new(tx.send(path)
            .map_err(|error| {
                println!("Could not send path: {}", error);
                // TODO: use a different error type. Eg failure
                hyper::Error::Status
            })
            .map(|_|{
                println!("Reply returned.");
                Response::new()
                    .with_header(ContentLength(PHRASE.len() as u64))
                    .with_body(PHRASE)
            })
        )
    }
}

/// Incredible simple hello world HTTP server.
/// Simply call with `curl localhost:6142`.
fn main() {
    let addr = "127.0.0.1:6142".parse().unwrap();

    let (tx, events) = channel(512);
    let server = Http::new()
        .bind(&addr, move || Ok(HelloWorld::new(tx.clone())))
        .unwrap();

    // Our state
    #[derive(Debug)]
    pub struct State {
        pub processed_events: u64,
    }
    let state = State { processed_events: 0 };

    // Start event processing in dedicated thread.
    let pool = CpuPool::new(1);
    let work = pool.spawn(
        events.fold(
            state,
            |mut state, path| {
                println!("Process Path: {}", path);
                // Let's make this computation artificially long.
                thread::sleep(time::Duration::from_secs(1));
                state.processed_events += 1;
                println!("Processed {} events.", state.processed_events);
                Ok(state)
            }
        )
    );

    // Uncomment to demonstarte borrow checker error.
    // state.processed_events += 1;

    println!("server running on localhost:6142");
    server.run().unwrap();
}
