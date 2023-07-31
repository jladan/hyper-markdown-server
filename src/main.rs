use std::convert::Infallible;
use std::sync::{Arc, RwLock};

use hyper::{Method, StatusCode, Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

use tera::Tera;

use hyper_markdown_server::{
    context::ServerContext,
    config::Config,
    uri,
    response,
    handler,
};

#[tokio::main]
async fn main() {
    // Load configuration
    let config = Config::build()
        .source_env()
        .build();
    let addr = config.addr;
    eprintln!("{config:#?}");

    // Set up templates
    let template_glob = config.template_dir.join("**/*.html");
    let tera = match Tera::new(&template_glob.to_str().unwrap()) {
        Ok(t) => RwLock::new(t),
        Err(e) => {eprintln!("{e}"); panic!()},
    };
    eprintln!("{tera:#?}");
    let context = Arc::new(ServerContext { config, tera });

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

async fn route(req: Request<Body>, state: Arc<ServerContext>) -> Result<Response<Body>, Infallible> {
    let resolved = uri::resolve(req.uri(), &state.config);
    eprintln!("{resolved:?}");
    match (req.method(), resolved) {
        (&Method::GET, Some(uri::Resolved::File(path))) => {
            Ok(response::send_file(&path).await)
        },
        (&Method::GET, Some(uri::Resolved::Markdown(path))) => {
            Ok(handler::markdown(&path, req.headers(), state.as_ref()).await)
        },
        (&Method::GET, Some(uri::Resolved::Directory(_path))) => {
            Ok(response::not_implemented())
        },
        (&Method::GET, None) => {
            Ok(response::not_found())
        },
        (&Method::HEAD, _) => {
            Ok(response::not_implemented())
        },
        _ => {
            Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::from("Only get requests are possible"))
                .unwrap())
        },
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install the CTRL+C signal handler");
}
