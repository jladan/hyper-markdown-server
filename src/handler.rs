//! Handlers for more complicated reqests
//!
//! For example, if GET Markdown is requested, then the headers are needed to determine the type of
//! response

use std::path::Path;

use tokio::fs;
use hyper::{Body, Response, HeaderMap};

use pulldown_cmark::{Parser, Options, html};

// use tera::Tera;

use crate::{
    context::ServerContext,
    response,
};

const MARKDOWN_TEMPLATE: &str = "markdown.html";

pub async fn markdown(path: &Path, headers: &HeaderMap, context: &ServerContext) -> Response<Body> {
    #[cfg(debug_assertions)]
    {
        //  Reload tera templates
        let mut lock = context.tera.write().expect("Could not open tera for reloading");
        match lock.full_reload() {
            Ok(_) => (),
            Err(e) => {eprintln!("{e}"); drop(lock); panic!()},
        }
    }
    let accepts = preferred_format(headers);
    for af in accepts {
        use AcceptFormat::*;
        match af {
            PartialHtml => return naked_markdown(path).await,
            Html | Any  => return full_markdown(path, context).await,
            _ => continue,
        }
    }
    response::not_acceptable()
}


async fn naked_markdown(path: &Path) -> Response<Body> {
    let contents = parse_markdown(path).await;
    match contents {
        Ok(contents) => {
            let body = Body::from(contents);
            return Response::new(body);
        },
        Err(_) => {
            //  TODO: better error handling of file errors
            return response::not_found();
        },
    }
}

async fn full_markdown(path: &Path, context: &ServerContext) -> Response<Body> {
    let contents = match parse_markdown(path).await {
        Ok(contents) => {
            contents
        },
        Err(_) => {
            //  TODO: better error handling of file errors
            return response::not_found();
        },
    };
    let tera = context.tera.read().unwrap();
    let mut context = tera::Context::new();
    context.insert("content", &contents);
    match tera.render(MARKDOWN_TEMPLATE, &context) {
        Ok(html_out) => {
            let body = Body::from(html_out);
            Response::new(body)
        },
        Err(e) => {
            eprintln!("{e}");
            response::server_error("Error in applying template")
        }
    }
}

async fn parse_markdown(path: &Path) -> Result<String, tokio::io::Error> {
    let contents = fs::read_to_string(path).await?;
    // NOTE(jladan): disable smart punctuation for sake of latex
    let options = Options::from_bits_truncate(0b1011110);
    let parser = Parser::new_ext(&contents, options);
    // TODO: Would there be any benefit to making this an async stream?
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    return Ok(html_out);
}

enum AcceptFormat {
    Html,
    PartialHtml,
    Json,
    Any,
}

fn preferred_format(headers: &HeaderMap) -> Vec<AcceptFormat> {
    if let Some(value) = headers.get("x-partial") {
        eprintln!("Partial header: {value:?}");
        return vec![AcceptFormat::PartialHtml];
    }
    if let Some(value) = headers.get("accept") {
        value.to_str().expect("accept header could not be converted to string?")
            .split(',').filter_map(|e| {
                if e.contains("json") {
                    Some(AcceptFormat::Json)
                } else if e.contains("html") {
                    Some(AcceptFormat::Html)
                } else if e.contains("*/*") {
                    Some(AcceptFormat::Any)
                } else {
                    None
                }
            }).collect()
    } else {
        vec![AcceptFormat::Any]
    }
}

