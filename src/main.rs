#[macro_use] extern crate failure;
extern crate futures;
extern crate futures_cpupool;
extern crate hyper;

use failure::Error;

use futures::Stream;
use futures::future::Future;
use futures::sink::Sink;
use futures::sync::mpsc::{channel, Sender};
use futures_cpupool::CpuPool;

use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};

use std::{str, thread, time};

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

    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let tx = self.sender.clone();
        let work = req.body().concat2()
            // Body to string
            .map_err(Error::from)
            .and_then(|v| -> Result<String, Error> {
                // TODO: Parse JSON.
                str::from_utf8(&v).map(String::from).map_err(Error::from)
            })
            // Process body
            .and_then(|body: String| { tx.send(body).map_err(Error::from) })
            .map_err(|error| {
                println!("Could not process event: {}", error);
                // TODO: use a different error type. Eg failure
                hyper::Error::Status
            })
            .map(|_|{
                println!("Reply returned.");
                Response::new()
                    .with_header(ContentLength(PHRASE.len() as u64))
                    .with_body(PHRASE)
            });
        Box::new(work)
    }
}

/// Incredible simple hello world HTTP server.
/// Simply call with `wrk -t24 -c99 -d10s -s post.lua http://localhost:6142/`.
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
    let state = State {
        processed_events: 0,
    };

    // Start event processing in dedicated thread.
    let pool = CpuPool::new(1);
    let work = pool.spawn(events.fold(state, |mut state, event| {
        println!("Process event: {}", event);
        // Let's make this computation artificially long.
        thread::sleep(time::Duration::from_secs(1));
        state.processed_events += 1;
        println!("Processed {} events.", state.processed_events);
        Ok(state)
    }));

    // Uncomment to demonstarte borrow checker error.
    // state.processed_events += 1;

    println!("server running on localhost:6142");
    server.run().unwrap();
    work.wait().unwrap();
}
