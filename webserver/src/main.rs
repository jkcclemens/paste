#![feature(plugin, custom_derive, macro_at_most_once_rep)]
#![plugin(rocket_codegen)]

extern crate base64;
extern crate chrono;
extern crate cookie;
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate failure;
extern crate git2;
extern crate hex;
#[macro_use]
extern crate if_chain;
extern crate ipnetwork;
#[macro_use]
extern crate lazy_static;
extern crate libflate;
extern crate percent_encoding;
extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;
extern crate reqwest;
extern crate rocket_contrib;
extern crate rocket;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate serde;
extern crate sidekiq;
extern crate sodiumoxide;
extern crate toml;
extern crate unicode_categories;
extern crate unicode_segmentation;
extern crate uuid;
extern crate xz2;

mod config;
mod database;
mod errors;
mod models;
mod redis_store;
mod routes;
mod sidekiq_;
mod store;
mod utils;

use routes::web::fairings;

use rocket::response::NamedFile;

use rocket_contrib::Template;

use std::env;

pub static SERVER_VERSION: Option<&'static str> = include!(concat!(env!("OUT_DIR"), "/version"));

lazy_static! {
  pub static ref RESOURCES_VERSION: Option<String> = git2::Repository::open(".")
    .and_then(|r| r.revparse_single("HEAD").map(|p| p.id()))
    .map(|r| r.to_string())
    .ok();
}

#[get("/")]
fn index() -> std::io::Result<NamedFile> {
  NamedFile::open("index.html")
}

fn main() {
  if !sodiumoxide::init() {
    println!("could not initialize libsodium");
    return;
  }

  dotenv::dotenv().ok();

  let config_path = match env::args().nth(1) {
    Some(p) => p,
    None => {
      println!("please specify the path to the configuration file as the first argument");
      return;
    }
  };

  let config = match config::load_config(&config_path) {
    Ok(c) => c,
    Err(e) => {
      println!("could not load config.toml: {}", e);
      return;
    }
  };

  #[get("/kek")]
  fn kek(user: routes::web::OptionalWebUser, sidekiq: rocket::State<sidekiq::Client>) {
    if let Some(u) = user.into_inner() {
      sidekiq.push(sidekiq_::Job::Email {
        name: u.name().to_string(),
        email: u.email().to_string(),
        subject: "Verify your email".into(),
        content: "pls do it bb".into(),
      }.into()).unwrap();
    }
  }

  rocket::ignite()
    .manage(database::init_pool())
    .manage(redis_store::init_pool())
    .manage(redis_store::init_sidekiq())
    .manage(config)
    .attach(fairings::SecurityHeaders)
    .attach(fairings::AntiCsrf)
    .attach(fairings::LastPage::default())
    .attach(Template::fairing())
    .catch(errors![
      routes::bad_request,
      routes::forbidden,
      routes::internal_server_error,
      routes::not_found,
    ])
    .mount("/", routes![
      kek,
      routes::web::index::get,

      routes::web::about::get,

      routes::web::auth::login::get,
      routes::web::auth::login::post,

      routes::web::auth::logout::post,

      routes::web::auth::register::get,
      routes::web::auth::register::post,

      routes::web::pastes::get::id,
      routes::web::pastes::get::username_id,
      routes::web::pastes::get::users_username_id,

      routes::web::pastes::files::raw::get,

      routes::web::pastes::get::edit,

      routes::web::pastes::post::post,

      routes::web::pastes::delete::delete,
      routes::web::pastes::patch::patch,

      routes::web::account::index::get,
      routes::web::account::index::patch,

      routes::web::account::keys::get,
      routes::web::account::keys::post,
      routes::web::account::keys::delete,

      routes::web::account::delete::get,
      routes::web::account::delete::delete,

      routes::web::users::get::get,
      routes::web::users::get::get_page,
    ])
    .mount("/static", routes!{
      routes::web::static_files::get,
    })
    .mount("/api/v0/pastes", routes![
      routes::pastes::get::get_all,
      routes::pastes::get::get_all_query,

      routes::pastes::post::post,
      routes::pastes::delete::delete,
      routes::pastes::get::get_query,
      routes::pastes::get::get,
      routes::pastes::patch::patch,

      routes::pastes::files::get::get,
      routes::pastes::files::patch::patch,
      routes::pastes::files::post::post,

      routes::pastes::files::individual::delete::delete,
      routes::pastes::files::individual::get::get,
      routes::pastes::files::individual::patch::patch,

      routes::pastes::files::individual::raw::get::get,
    ])
    .launch();
}