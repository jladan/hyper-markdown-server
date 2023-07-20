use std::convert::Infallible;
use std::sync::Arc;

use hyper::{Method, StatusCode, Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

use hyper_markdown_server::{
    config::Config,
    uri,
};

#[tokio::main]
async fn main() {
    let config = Config::build()
        .source_env()
        .build();
    let addr = config.addr;
    let context = Arc::new(ServerContext { config });

    // A `Service` is needed for every connection.
    // This creates one from the `route` function.
    let make_svc = make_service_fn(move |_conn| {
        // NOTE: the state must be cloned before use in an async block
        // This clone is for the function which makes new services
        let context = context.clone();
        let service = service_fn(move |req| {
            // NOTE: the state must be cloned a second time
            // This clone is for the service itself
            route(req, context.clone())
        });
        async move {
            Ok::<_, Infallible>(service)
        }
    });

    let server = Server::bind(&addr).serve(make_svc);
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
}

struct ServerContext {
    config: Config,
}

async fn route(req: Request<Body>, state: Arc<ServerContext>) -> Result<Response<Body>, Infallible> {
    let resolved = uri::resolve(req.uri(), &state.config);
    eprintln!("{resolved:?}");
    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, _) => {
            Response::builder()
                .status(StatusCode::NOT_IMPLEMENTED)
                .body(Body::from("not yet implemented"))
                .unwrap()
        },
        (&Method::HEAD, _) => {
            Response::builder()
                .status(StatusCode::NOT_IMPLEMENTED)
                .body(Body::from("not yet implemented"))
                .unwrap()
        },
        _ => {
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
