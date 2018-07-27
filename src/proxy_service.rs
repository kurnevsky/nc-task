use std::collections::HashMap;
use std::error::Error as StdError;
use std::sync::Arc;

use futures::{Future, future};
use hyper::{Body, Client, Error, Request, Response, StatusCode};
use hyper::client::HttpConnector;
use hyper::service::{NewService, Service};
use parking_lot::RwLock;

pub struct ProxyService {
    pub client: Client<HttpConnector, Body>,
    pub routes: Arc<RwLock<HashMap<String, String>>>,
}

impl Service for ProxyService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Error;
    type Future = Box<Future<Item = Response<Body>, Error = Self::Error> + Send>;

    fn call(&mut self, mut req: Request<Self::ReqBody>) -> Self::Future {
        if let Some(uri) = self.routes.read().get(req.uri().path()) {
            *req.uri_mut() = uri.parse().unwrap();
            Box::new(self.client.request(req))
        } else {
            Box::new(future::ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap()))
        }
    }
}

pub struct ProxyNewService {
    pub client: Client<HttpConnector, Body>,
    pub routes: Arc<RwLock<HashMap<String, String>>>,
}

impl NewService for ProxyNewService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Error;
    type Service = ProxyService;
    type Future = Box<Future<Item = Self::Service, Error = Self::InitError> + Send>;
    type InitError = Box<StdError + Send + Sync>;

    fn new_service(&self) -> Self::Future {
        Box::new(future::ok(ProxyService {
            client: self.client.clone(),
            routes: self.routes.clone(),
        }))
    }
}
