use hyper::Server;
use std::net::SocketAddr;
use http_handler::HttpArgs;
use structopt::StructOpt;
use futures::prelude::*;

mod token;
mod http_handler;

fn main() {

    let args = HttpArgs::from_args();

    let addr = SocketAddr::new("127.0.0.1".parse().unwrap(), args.port());

    // And a MakeService to handle each connection...
    let make_service = move || {
        args.http()
    };

    let server = Server::bind(&addr)
        .http1_only(true)
        .http1_max_buf_size(8192)
        .tcp_nodelay(true)
        .serve(make_service)
        .map_err(|e| {
            eprintln!("server error: {}", e);
        });

    hyper::rt::run(server);
}
