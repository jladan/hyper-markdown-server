//! Handlers for more complicated reqests
//!
//! For example, if GET Markdown is requested, then the headers are needed to determine the type of
//! response

use std::{path::Path, ops::Deref, str::FromStr};

use tokio::fs;
use hyper::{Body, Response, HeaderMap, http::HeaderValue};

use pulldown_cmark::{Parser, Options, html};

// use tera::Tera;

use crate::{
    context::ServerContext,
    response,
};

const MARKDOWN_TEMPLATE: &str = "markdown.html";

pub fn directory(path: &Path, headers: &HeaderMap, _context: &ServerContext) -> Response<Body> {
    let accepts = preferred_format(headers);
    use  AcceptFormat::*;
    for af in accepts {
        match af {
            PartialHtml => return dir_html(path, _context, true),
            Html | Any  => return dir_html(path, _context, false),
            _ => continue,
        }
    }
    response::not_acceptable()
}

fn dir_html(path: &Path, context: &ServerContext, partial: bool) -> Response<Body> {
    let tera = context.tera.read().expect("could not read template engine");
    if !partial {
        context.refresh_roottree();
    }
    let root_tree = context.roottree.read().expect("could not read web-root tree");
    let dirtree = crate::context::walk_dir(path, false).expect("failure to trace directory");
    let mut context = tera::Context::new();
    context.insert("dirtree", &root_tree.deref());
    context.insert("dir_contents", &dirtree);
    let rendered = if partial {
        tera.render("directory-chunk.html", &context)
    } else {
        tera.render("directory.html", &context)
    };
    match rendered {
        Ok(contents) => response::send_html(contents),
        Err(e) => {eprintln!("{e}"); response::server_error("")},
    }
}

// General files {{{

pub async fn file(path: &Path, headers: &HeaderMap, context: &ServerContext) -> Response<Body> {
    let accepts = preferred_format(headers);
    for af in accepts {
        use AcceptFormat::*;
        match af {
            PartialHtml => return wrapped_file(path, context).await,
            _ => continue,
        }
    }
    response::send_file(path).await
}

async fn wrapped_file(path: &Path, _context: &ServerContext) -> Response<Body> {
    let stripped = _context.strip_path(path);
    if stripped.is_none() {
        return response::not_found();
    }
    let stripped = stripped.unwrap();
    let guess = mime_guess::from_path(path).first();
    match guess {
        Some(mime) => {
            match mime.type_().as_str() {
                "image" => {
                    response::send_html(format!("<img src=\"{}\" />", stripped.to_string_lossy()))
                },
                "video" => Response::new(Body::from("this is a video")),
                "application" => {
                    if mime.subtype() == "pdf" {
                        response::send_html(format!("<iframe src=\"{}\" />", stripped.to_string_lossy()))
                    } else {
                        Response::new(Body::from("some application"))

                    }
                },
                _ => response::send_file(path).await,
            }

        },
        None => response::send_file(path).await,
    }
}

// }}}

// Markdown handlers {{{
pub async fn markdown(path: &Path, headers: &HeaderMap, context: &ServerContext) -> Response<Body> {
    #[cfg(debug_assertions)]
    context.reload_templates();

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
            return response::send_html(contents);
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
    // Reload roottree
    context.refresh_roottree();
    let dirtree = context.roottree.read().expect("Could not read web root");
    let tera = context.tera.read().unwrap();
    let mut context = tera::Context::new();
    context.insert("content", &contents);
    context.insert("dirtree", &dirtree.deref());
    match tera.render(MARKDOWN_TEMPLATE, &context) {
        Ok(html_out) => {
            response::send_html(html_out)
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

// }}}

// Determining accepted format {{{
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

// }}}
