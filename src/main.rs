mod routes;
mod models;
mod utils;

use actix_web::{web, App, HttpServer};
use rust_embed::RustEmbed;
use tera::Tera;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut tera = Tera::default();
    for file in Templates::iter() {
        if let Some(content) = Templates::get(file.as_ref()) {
            let content_str = std::str::from_utf8(&content.data).unwrap();
            tera.add_raw_template(file.as_ref(), content_str).unwrap();
        }
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .configure(routes::init)
    })
    .bind("0.0.0.0:80")?
    .run()
    .await
}
