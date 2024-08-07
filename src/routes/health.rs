use actix_web::{HttpResponse, Responder};

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

pub async fn readiness_check() -> impl Responder {
    HttpResponse::Ok().body("Ready")
}
