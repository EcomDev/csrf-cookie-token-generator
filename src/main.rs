use hyper::{Server, Request, Body};
use std::net::SocketAddr;
use http::HttpArgs;
use structopt::StructOpt;
use hyper::service::service_fn_ok;
use crate::http::Http;
use futures::prelude::*;

mod token;
mod http;

fn main() {
    let args = HttpArgs::from_args();

    let addr = SocketAddr::new("127.0.0.1".parse().unwrap(), args.port());

    let handler: Http = args.http();

    // And a MakeService to handle each connection...
    let make_service = move || {
        let handler = handler.clone();
        service_fn_ok( move | req: Request<Body>| handler.serve(req) )
    };

    let server = Server::bind(&addr)
        .serve(make_service)
        .map_err(|e| {
            eprintln!("server error: {}", e);
        });

    hyper::rt::run(server);
}
