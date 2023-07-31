//! Response-building functions
//!
//! Mostly just errors, but also sending files
//!
//!
use std::path::Path;

use tokio::fs::{self, File};
use tokio_util::codec::{BytesCodec, FramedRead};

use hyper::{StatusCode, Body, Response};

use pulldown_cmark::{Parser, Options, html};

use crate::context::ServerContext;

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

pub async fn send_markdown(path: &Path, _headers: &hyper::HeaderMap, _context: &ServerContext) -> Response<Body> {
    let contents = fs::read_to_string(path).await;
    if let Ok(contents) = contents {
        // NOTE(jladan): disable smart punctuation for sake of latex
        let options = Options::from_bits_truncate(0b1011110);
        let parser = Parser::new_ext(&contents, options);
        // TODO: Would there be any benefit to making this an async stream?
        let mut html_out = String::new();
        html::push_html(&mut html_out, parser);
        let body = Body::from(html_out);
        return Response::new(body);
    } else {
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

