use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use hyper::{Method, StatusCode, Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0,0,0,0], 7878));
    let state = Arc::new(ServerState { counter: AtomicU64::new(0) });

    // A `Service` is needed for every connection.
    // This creates one from the `route` function.
    let make_svc = make_service_fn(move |_conn| {
        // NOTE: the state must be cloned before use in an async block
        let state = state.clone();
        async move {
        Ok::<_, Infallible>(service_fn(move |req| {
            // NOTE: the state must be cloned AGAIN?
            let state = state.clone();
            async move {route(req, state).await}
        }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
}

struct ServerState {
    counter: AtomicU64,
}

async fn route(req: Request<Body>, state: Arc<ServerState>) -> Result<Response<Body>, Infallible> {
    state.counter.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    let count = &state.counter;
    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, _) => {
            eprintln!("{count:?}, refs: {}\n {:#?}", Arc::strong_count(&state), req);
            Response::builder()
                .status(StatusCode::NOT_IMPLEMENTED)
                .body(Body::from("not yet implemented"))
                .unwrap()
        },
        (&Method::HEAD, _) => {
            eprintln!("{count:?}, {:#?}", req);
            Response::builder()
                .status(StatusCode::NOT_IMPLEMENTED)
                .body(Body::from("not yet implemented"))
                .unwrap()
        },
        _ => {
            eprintln!("{:#?}", req);
            Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::from("Only get requests are possible"))
                .unwrap()
        },
    };

    Ok(response)
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install the CTRL+C signal handler");
}
