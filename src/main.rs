use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::{Method, StatusCode, Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

use futures::{TryStreamExt as _, Stream};

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0,0,0,0], 7878));

    // A `Service` is needed for every connection.
    // This creates one from the `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(echo))
    });

    let server = Server::bind(&addr).serve(make_svc);
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
}

async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            eprintln!("{:#?}", req);
            *response.body_mut() = Body::from("Try POSTing data to /echo");
        },
        (&Method::POST, "/echo") => {
            *response.body_mut() = req.into_body();
        },
        (&Method::POST, "/echo/reverse") => {
            // Restrict large payloads
            // NOTE(jladan): This line is different than in the tutorial, because `sizehint`
            // returns a tuple rather than the `SizeHint` struct suggested by the docs
            eprintln!("{:#?}", req);
            let upper = req.body().size_hint().1
                .unwrap_or(usize::MAX);
            if upper > 1024 * 64 {
                eprintln!("size hint: {:?}", req.body().size_hint());
                let mut resp = Response::new(Body::from("Body too big"));
                *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
                return Ok(resp);
            }
            // Await the full body
            let full_body = hyper::body::to_bytes(req.into_body()).await?;

            // reverse the bytes
            let reversed = full_body.iter()
                .rev()
                .cloned()
                .collect::<Vec<u8>>();

            *response.body_mut() = reversed.into();
        },
        (&Method::POST, "/echo/uppercase") => {
            let mapping = req
                .into_body()
                .map_ok(|chunk| {
                    chunk.iter()
                        .map(|byte| byte.to_ascii_uppercase())
                        .collect::<Vec<u8>>()
                });

            *response.body_mut() = Body::wrap_stream(mapping);
        },
        _ => *response.status_mut() = StatusCode::NOT_FOUND,
    };

    Ok(response)
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install the CTRL+C signal handler");
}

async fn hello_wold(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("Hello, World".into()))
}
