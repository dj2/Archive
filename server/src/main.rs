use tokio::fs::File;
use tokio_util::codec::{ BytesCodec, FramedRead };
use hyper::{ Body, Request, Response, Result, Server, StatusCode };
use hyper::header::{ self, HeaderValue };
use routerify::prelude::*;
use routerify::{ Middleware, Router, RouterService, RequestInfo };
use std::env;
use std::net::{ IpAddr, Ipv4Addr, SocketAddr };
use std::path::{ Path, PathBuf };
use tree_magic;

static SERVER_DEFAULT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
static SERVER_DEFAULT_PORT: u16 = 3000;

static NOTFOUND: &[u8] = b"Not Found";

struct State<'a> {
  asset_path: &'a Path,
  data_path: &'a Path,
}

#[tokio::main]
async fn main() {
  let addr = SocketAddr::new(
      env::var("HOST").ok()
          .and_then(|host| host.parse::<IpAddr>().ok())
          .unwrap_or(SERVER_DEFAULT_IP),
      env::var("PORT").ok()
          .and_then(|port| port.parse::<u16>().ok())
          .unwrap_or(SERVER_DEFAULT_PORT));

  let router = router();
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

fn router() -> Router<Body, hyper::Error> {
  let state = State {
    asset_path: Path::new("/tmp/assets"),
    data_path: Path::new("/tmp/data"),
  };

  Router::builder()
      .data(state)
      .middleware(Middleware::pre(logger_middleware))
      .middleware(Middleware::post(server_middleware))
      .get("/", index_handler)
      .get("/index.html", index_handler)
      .get("/assets/*", asset_handler)
      .err_handler_with_info(error_handler)
      .build()
      .unwrap()
}

async fn server_middleware(mut res: Response<Body>) -> Result<Response<Body>> {
    res.headers_mut()
        .insert(header::SERVER, HeaderValue::from_str("Archive").unwrap());

    if !res.headers().contains_key(header::CONTENT_TYPE) {
      res.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_str("text/html").unwrap());
    }

    Ok(res)
}

async fn logger_middleware(req: Request<Body>) -> Result<Request<Body>> {
  println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
  Ok(req)
}

async fn error_handler(err: routerify::Error, _: RequestInfo) -> Response<Body> {
  eprintln!("{}", err);

  Response::builder()
    .status(StatusCode::INTERNAL_SERVER_ERROR)
    .body(Body::from(format!("Something went wrong: {}", err)))
    .unwrap()
}

async fn index_handler(_req: Request<Body>) -> Result<Response<Body>> {
  Ok(Response::new(Body::from("Index")))
}

async fn asset_handler(req: Request<Body>) -> Result<Response<Body>> {
  let state = req.data::<State>().unwrap();

  let mut s: String = req.uri().path()[7..].to_string();
  if !s.starts_with('/') {
    s = format!("/{}", s);
  }

  let mut path = PathBuf::from(state.asset_path);
  for p in s.split('/') {
    if p == ".." || p == "." {
      continue;
    }
    path.push(p);
  }
  let path = path.canonicalize().ok();
  if let Some(p) = path {
    send_file(&p).await
  } else {
    Ok(not_found())
  }
}

async fn send_file(path: &Path) -> Result<Response<Body>> {
  if let Ok(file) = File::open(path).await {
      let ty = tree_magic::from_filepath(path);
      let stream = FramedRead::new(file, BytesCodec::new());
      let body = Body::wrap_stream(stream);

      let mut resp = Response::new(body);
      resp.headers_mut().insert(header::CONTENT_TYPE,
              HeaderValue::from_str(&ty).unwrap());
      return Ok(resp);
  }

  Ok(not_found())
}

fn not_found() -> Response<Body> {
  Response::builder()
      .status(StatusCode::NOT_FOUND)
      .body(NOTFOUND.into())
      .unwrap()
}
