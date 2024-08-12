use actix_web::{web, HttpRequest, HttpResponse, Responder};
use tera::Tera;

use crate::models::{
    query_params::{FuelOptionQueryParams, IndexQueryParams},
    state::ApplicationState,
};

use super::calculations;

pub async fn fuel(
    query: web::Query<IndexQueryParams>,
    req: HttpRequest,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let mut ctx = tera::Context::new();
    let (app_state, query) = ApplicationState::from_query_params(query.into_inner());

    match query.submit {
        Some(s) if s == "Vorige" => {
            app_state.apply("input", &mut ctx);
            let rendered = tmpl.render("wb_form.html", &ctx).unwrap();
            return HttpResponse::Ok().content_type("text/html").body(rendered);
        }
        _ => (),
    }

    calculations::render_calculations(&app_state, &mut ctx, req, tmpl, "calculations_form.html")
}

pub async fn fuel_option(
    query: web::Query<FuelOptionQueryParams>,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let mut ctx = tera::Context::new();
    let query_params = query.into_inner();

    if let Some(fm) = query_params.fuel_max {
        ctx.insert("fuel_max", &fm);
    }

    if let Some(ft) = query_params.fuel_type {
        ctx.insert("fuel_type", &ft);
    }
    let rendered = tmpl.render("fuel_max_fuel_option.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}
