use std::collections::HashMap;
use std::error::Error as StdError;
use std::ops::Deref;
use std::sync::Arc;

use futures::{Future, Stream, future};
use hyper::{Body, Error, Method, Request, Response, StatusCode};
use hyper::service::{NewService, Service};
use parking_lot::RwLock;
use serde_json;

pub struct ProxyEditorService {
    pub routes: Arc<RwLock<HashMap<String, String>>>,
}

impl Service for ProxyEditorService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Error;
    type Future = Box<Future<Item = Response<Body>, Error = Self::Error> + Send>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => {
                let json = serde_json::to_string(self.routes.read().deref()).unwrap();
                Box::new(future::ok(Response::new(Body::from(json))))
            },
            (&Method::PUT, _) => {
                let routes = self.routes.clone();
                let path = req.uri().path().to_string();
                Box::new(req.into_body().concat2().and_then(move |chunk| {
                    if let Ok(body) = String::from_utf8(chunk.to_vec()) {
                        routes.write().insert(path, body);
                        Box::new(future::ok(Response::new(Body::empty())))
                    } else {
                        Box::new(future::ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::empty())
                            .unwrap()))
                    }
                }))
            },
            (&Method::DELETE, _) => {
                if let Some(_) = self.routes.write().remove(req.uri().path()) {
                    Box::new(future::ok(Response::new(Body::empty())))
                } else {
                    Box::new(future::ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Body::empty())
                        .unwrap()))
                }
            },
            _ => {
                Box::new(future::ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .unwrap()))
            }
        }
    }
}

pub struct ProxyEditorNewService {
    pub routes: Arc<RwLock<HashMap<String, String>>>,
}

impl NewService for ProxyEditorNewService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Error;
    type Service = ProxyEditorService;
    type Future = Box<Future<Item = Self::Service, Error = Self::InitError> + Send>;
    type InitError = Box<StdError + Send + Sync>;

    fn new_service(&self) -> Self::Future {
        Box::new(future::ok(ProxyEditorService {
            routes: self.routes.clone(),
        }))
    }
}
