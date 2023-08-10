//! Response-building functions
//!
//! Mostly just errors, but also sending files
//!
//!
use std::path::Path;

use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use hyper::{StatusCode, Body, Response, http::HeaderValue};

/// Stream a chunked file, with Content-Type guessed from file extension
pub async fn send_file(resolved: &Path) -> Response<Body> {
    let fresult = File::open(resolved).await;
    if let Ok(file) = fresult {
        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(stream);
        let mut resp = Response::new(body);
        // Now add content-type
        let guess = mime_guess::from_path(resolved).first();
        if let Some(mime) =  guess {
            resp.headers_mut().append("Content-Type", HeaderValue::from_str(mime.essence_str()).unwrap());
        }
        resp
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

pub fn server_error(msg: &str) -> Response<Body> {
    Response::builder()
       .status(StatusCode::INTERNAL_SERVER_ERROR)
       .body(Body::from(msg.to_string()))
       .unwrap()
}

pub fn not_acceptable() -> Response<Body> {
    Response::builder()
       .status(StatusCode::NOT_ACCEPTABLE)
       .body(Body::from("Requested format can not be provided"))
       .unwrap()
}

