mod archive;

use rocket_contrib::templates::Template;
use rocket::{Request, State};
use rocket::http::ContentType;
use rocket::response::{content, NamedFile};
use std::path::PathBuf;
use std::collections::HashMap;
use rocket_contrib::serve::{crate_relative, StaticFiles};
use archive::Archive;
use tokio::sync::RwLock;
use tokio::io::AsyncReadExt;
use std::env;

#[macro_use]
extern crate rocket;

#[get("/asset/<file..>")]
async fn assets(file: PathBuf, state: State<'_, ArchiveState>) -> Option<NamedFile> {
    let archive = state.archive.read().await;
    archive.retrieve_asset(file).await
}

#[derive(serde::Serialize)]
struct NoteContext<'a> {
    id: &'a str,
    content: &'a str,
    parent: &'static str,
}
#[get("/note/<name..>", rank = 2, format = "text/html")]
async fn note_html(name: PathBuf, state: State<'_, ArchiveState>) -> Template {
    let archive = state.archive.read().await;
    let id = name.to_str().unwrap().to_string();

    let mut file = archive.retrieve_note(name).await.unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).await;

    let ctx = NoteContext {
        id: &id,
        content: &buf,
        parent: "layout",
    };
    Template::render("show", &ctx)
}

#[get("/note/<name..>", rank = 1, format = "text/plain")]
async fn note_plain(name: PathBuf, state: State<'_, ArchiveState>) -> content::Content<NamedFile> {
    let archive = state.archive.read().await;
    content::Content(ContentType::Plain,
                    archive.retrieve_note(name).await.unwrap())
}

#[derive(serde::Serialize)]
struct IndexContext {
    parent: &'static str,
}
#[get("/")]
async fn index<'a>() -> Template {
    let ctx = IndexContext {
        parent: "layout",
    };
    Template::render("index", &ctx)
}

#[catch(404)]
fn not_found(req: &Request<'_>) -> Template {
    let mut map = HashMap::new();
    map.insert("path", req.uri().path());

    Template::render("404", &map)
}

struct ArchiveState {
    archive: RwLock<Archive>,
}

static SERVER_DEFAULT_ASSET_PATH: &str = "./data/assets";
static SERVER_DEFAULT_DATA_PATH: &str = "./data/data";

#[launch]
fn rocket() -> rocket::Rocket {
    let rocket = rocket::ignite()
        .attach(Template::fairing())
        .register(catchers![not_found])
        .mount("/", StaticFiles::from(crate_relative!("public")))
        .mount("/", routes![assets, index])
        .mount("/", routes![note_plain, note_html]);

    let asset_path =
        env::var("ARCHIVE_ASSET_PATH").unwrap_or_else(|_| SERVER_DEFAULT_ASSET_PATH.to_string());
    let data_path =
        env::var("ARCHIVE_DATA_PATH").unwrap_or_else(|_| SERVER_DEFAULT_DATA_PATH.to_string());

    let archive = Archive::new(&data_path, &asset_path);

    rocket.manage(ArchiveState { archive: RwLock::new(archive) })
}
