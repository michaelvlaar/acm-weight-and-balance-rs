mod routes;
mod models;
mod utils;

use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse, Responder};
use rust_embed::RustEmbed;
use tera::Tera;
use mime_guess::from_path;
use tokio;

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

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
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

    let tera_clone = tera.clone();

    let main_server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera_clone.clone()))
            .route("/assets/{filename:.*}", web::get().to(serve_asset))
            .configure(routes::init)
    })
    .bind("0.0.0.0:80")?;

    // Health check server
    let health_server = HttpServer::new(|| {
        App::new()
            .route("/healthz", web::get().to(health_check))
    })
    .bind("0.0.0.0:8081")?; // Different port for health checks

    tokio::try_join!(main_server.run(), health_server.run())?;
    Ok(())
}
