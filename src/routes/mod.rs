use actix_web::web;

mod calculations;
mod export;
mod print;
mod index;
mod health;
mod fuel;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/", web::get().to(index::index))
        .route("/wind-option", web::get().to(index::wind_option))
        .route("/fuel", web::get().to(fuel::fuel))
        .route("/fuel-option", web::get().to(fuel::fuel_option))
        .route("/calculations", web::get().to(calculations::calculations))
        .route("/perf-tod", web::get().to(calculations::perf_tod))
        .route("/perf-ldr", web::get().to(calculations::perf_ldr))
        .route("/wb-chart", web::get().to(calculations::wb_chart))
        .route("/wb-table", web::get().to(calculations::wb_table))
        .route("/export", web::get().to(export::export))
        .route("/print", web::get().to(print::print))
        .route("/health", web::get().to(health::health_check))
        .route("/ready", web::get().to(health::readiness_check));
}

