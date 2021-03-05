use hyper::header::{self, HeaderValue};
use hyper::{Body, Request, Response, Result, Server, StatusCode};
use routerify::prelude::*;
use routerify::{Middleware, RequestInfo, Router, RouterService};
use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio_util::codec::{BytesCodec, FramedRead};
use handlebars::Handlebars;
use std::collections::HashMap;

#[macro_use]
extern crate lazy_static;

static SERVER_DEFAULT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
static SERVER_DEFAULT_PORT: u16 = 3000;
static SERVER_DEFAULT_ASSET_PATH: &str = "./data/assets";
static SERVER_DEFAULT_DATA_PATH: &str = "./data/data";

static NOTFOUND: &str = "Not Found";

lazy_static! {
  static ref HB: Handlebars<'static> = {
    let mut hb = Handlebars::new();
    if let Err(e) = hb.register_template_file("index", "./server/views/index.hbs") {
      panic!("Failed to load template: index\n{}", e);
    }
    if let Err(e) = hb.register_template_file("layout", "./server/views/layout.hbs") {
      panic!("Failed to load template: layout\n{}", e);
    }
    hb
  };
}

struct State<'a> {
    asset_path: String,
    data_path: String,
    hb: &'a Handlebars<'a>,
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::new(
        env::var("ARCHIVE_HOST")
            .ok()
            .and_then(|host| host.parse::<IpAddr>().ok())
            .unwrap_or(SERVER_DEFAULT_IP),
        env::var("ARCHIVE_PORT")
            .ok()
            .and_then(|port| port.parse::<u16>().ok())
            .unwrap_or(SERVER_DEFAULT_PORT),
    );

    let asset_path =
        env::var("ARCHIVE_ASSET_PATH").unwrap_or_else(|_| SERVER_DEFAULT_ASSET_PATH.to_string());
    let data_path =
        env::var("ARCHIVE_DATA_PATH").unwrap_or_else(|_| SERVER_DEFAULT_DATA_PATH.to_string());

    let router = router(asset_path, data_path);
    let service = RouterService::new(router).unwrap();

    let server = Server::bind(&addr).serve(service);
    let server = server.with_graceful_shutdown(shutdown());

    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}

async fn shutdown() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install ctrl-c signal handler");
}

async fn server_middleware(mut res: Response<Body>) -> Result<Response<Body>> {
    res.headers_mut()
        .insert(header::SERVER, HeaderValue::from_str("Archive").unwrap());

    if !res.headers().contains_key(header::CONTENT_TYPE) {
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str("text/html").unwrap(),
        );
    }

    Ok(res)
}

async fn logger_middleware(req: Request<Body>) -> Result<Request<Body>> {
    println!(
        "{} {} {}",
        req.remote_addr(),
        req.method(),
        req.uri().path()
    );
    Ok(req)
}

async fn error_handler(err: routerify::Error, _: RequestInfo) -> Response<Body> {
    eprintln!("{}", err);

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong: {}", err)))
        .unwrap()
}

async fn index_handler(req: Request<Body>) -> Result<Response<Body>> {
    let state = req.data::<State>().unwrap();

    let mut d = HashMap::new();
    d.insert("parent", "layout");

    let body = state.hb.render("index", &d).ok();
    if let Some(body) = body {
      Ok(Response::new(Body::from(body)))
    } else {
      Ok(not_found())
    }
}

async fn static_handler(req: Request<Body>) -> Result<Response<Body>> {
    let path = clean_path("server/public", &req.uri().path());
    if let Some(path) = path {
        return send_file(&path).await;
    }
    Ok(not_found())
}

async fn asset_handler(req: Request<Body>) -> Result<Response<Body>> {
    let state = req.data::<State>().unwrap();
    let path = clean_path(&state.asset_path, &req.uri().path()[7..]);
    if let Some(path) = path {
        return send_file(&path).await;
    }
    Ok(not_found())
}

async fn note_handler(req: Request<Body>) -> Result<Response<Body>> {
    let state = req.data::<State>().unwrap();
    let path = clean_path(&state.data_path, &req.uri().path()[6..]);
    if let Some(path) = path {
        if let Ok(mut file) = File::open(path).await {
            let mut body = Vec::new();
            if file.read_to_end(&mut body).await.is_ok() {
                let body = String::from_utf8_lossy(&body).into_owned();
                let mut resp = Response::new(Body::from(body));
                resp.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("text/html").unwrap(),
                );
                return Ok(resp);
            }
        }
    }
    Ok(not_found())
}

async fn send_file(path: &Path) -> Result<Response<Body>> {
    if let Ok(file) = File::open(path).await {
        let ty = tree_magic::from_filepath(path);
        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(stream);

        let mut resp = Response::new(body);
        resp.headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_str(&ty).unwrap());
        return Ok(resp);
    }

    Ok(not_found())
}

fn router(asset_path: String, data_path: String) -> Router<Body, hyper::Error> {
  let state = State {
        asset_path,
        data_path,
        hb: &HB,
    };

    Router::builder()
        .data(state)
        .middleware(Middleware::pre(logger_middleware))
        .middleware(Middleware::post(server_middleware))
        .get("/css/*", static_handler)
        .get("/js/*", static_handler)
        .get("/notes/:name", note_handler)
        .get("/assets/*", asset_handler)
        .get("/index.html", index_handler)
        .get("/", index_handler)
        .err_handler_with_info(error_handler)
        .build()
        .unwrap()
}

fn clean_path(prefix: &str, path: &str) -> Option<PathBuf> {
    let mut s = path.to_string();
    if !s.starts_with('/') {
        s = format!("/{}", s);
    }

    let mut path = PathBuf::from(prefix);
    for p in s.split('/') {
        if p == ".." || p == "." {
            continue;
        }
        path.push(p);
    }
    path.canonicalize().ok()
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOTFOUND.into())
        .unwrap()
}
