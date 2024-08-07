use actix_web::web;

mod performance;
mod export;
mod print;
mod index;
mod health;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg
        .route("/", web::get().to(index::index))
        .route("/fuel-option", web::get().to(index::fuel_option))
        .route("/wind-option", web::get().to(index::wind_option))
        .route("/performance", web::get().to(performance::performance))
        .route("/perf-tod", web::get().to(performance::perf_tod))
        .route("/perf-ldr", web::get().to(performance::perf_ldr))
        .route("/wb-chart", web::get().to(performance::wb_chart))
        .route("/wb-table", web::get().to(performance::wb_table))
        .route("/export", web::get().to(export::export))
        .route("/print", web::get().to(print::print))
        .route("/health", web::get().to(health::health_check))
        .route("/ready", web::get().to(health::readiness_check));
}

