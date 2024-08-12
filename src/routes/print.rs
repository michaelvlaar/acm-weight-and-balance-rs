use actix_web::{web, HttpRequest, Responder};
use tera::Tera;

use crate::models::{query_params::IndexQueryParams, state::ApplicationState};

use super::calculations::render_calculations;

pub async fn print(
    query: web::Query<IndexQueryParams>,
    req: HttpRequest,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let mut ctx = tera::Context::new();
    let (app_state, query) = ApplicationState::from_query_params(query.into_inner());

    ctx.insert("print", &true);
    ctx.insert("document_reference", &query.reference);

    render_calculations(&app_state, &mut ctx, req, tmpl, "print.html")
}
