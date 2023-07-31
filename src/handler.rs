//! Handlers for more complicated reqests
//!
//! For example, if GET Markdown is requested, then the headers are needed to determine the type of
//! response

use std::path::Path;

use tokio::fs;
use hyper::{Body, Response, HeaderMap};

use pulldown_cmark::{Parser, Options, html};

use crate::{
    context::ServerContext,
    response,
};

pub async fn markdown(path: &Path, _headers: &HeaderMap, _context: &ServerContext) -> Response<Body> {
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
        return response::not_found();
    }
}

