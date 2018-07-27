extern crate futures;
extern crate http;
extern crate hyper;
extern crate parking_lot;
extern crate serde_json;

mod proxy_service;
mod proxy_editor_service;

use std::collections::HashMap;
use std::sync::Arc;

use futures::Future;
use hyper::{Client, Server};
use parking_lot::RwLock;

use proxy_service::*;
use proxy_editor_service::*;

fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();
    let addr2 = ([127, 0, 0, 1], 3002).into();

    let routes = Arc::new(RwLock::new(HashMap::new()));

    let proxy_service = ProxyNewService {
        client: Client::new(),
        routes: routes.clone(),
    };
    let proxy_editor_service = ProxyEditorNewService {
        routes,
    };

    let proxy_server = Server::bind(&addr).serve(proxy_service);
    let proxy_editor_server = Server::bind(&addr2).serve(proxy_editor_service);

    let server = proxy_server
        .join(proxy_editor_server)
        .map(|_| ())
        .map_err(|e| eprintln!("Server error: {}", e));

    hyper::rt::run(server);
}
