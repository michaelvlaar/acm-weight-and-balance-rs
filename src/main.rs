mod routes;
mod models;
mod utils;

use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, Responder};
use rust_embed::RustEmbed;
use tera::Tera;
use mime_guess::from_path;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

async fn serve_asset(req: HttpRequest) -> impl Responder {
    let path: String = req.match_info().query("filename").parse().unwrap();

    match Assets::get(&path) {
        Some(content) => {
            let body = content.data;
            let mime_type = from_path(path).first_or_octet_stream();
            HttpResponse::Ok()
                .content_type(mime_type.as_ref())
                .body(body)
        },
        None => HttpResponse::NotFound().body("Asset not found"),
    }
}

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
            .route("/assets/{filename:.*}", web::get().to(serve_asset))
    })
    .bind("0.0.0.0:80")?
    .run()
    .await
}
