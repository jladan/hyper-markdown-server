//! Response-building functions
//!
//! Mostly just errors, but also sending files
//!
//!
use std::path::Path;

use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use hyper::{StatusCode, Body, Response};

pub async fn send_file(resolved: &Path) -> Response<Body> {
    let fresult = File::open(resolved).await;
    if let Ok(file) = fresult {
        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(stream);
        return Response::new(body);
    } else {
        // TODO: Probably want to handle the different types of file errors here
        return not_found();
    }
}

pub fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("File not found"))
        .unwrap()
}

pub fn not_implemented() -> Response<Body> {
    Response::builder()
       .status(StatusCode::NOT_IMPLEMENTED)
       .body(Body::from("not yet implemented"))
       .unwrap()
}

