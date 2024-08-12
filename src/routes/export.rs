use actix_web::{web, HttpRequest, HttpResponse, Responder};
use tera::Tera;

use crate::models::{query_params::IndexQueryParams, state::ApplicationState};

use super::calculations::render_calculations;

pub async fn export(
    query: web::Query<IndexQueryParams>,
    req: HttpRequest,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let mut ctx = tera::Context::new();
    let (app_state, query) = ApplicationState::from_query_params(query.into_inner());

    match query.submit {
        Some(s) if s == "Vorige" => {
            return render_calculations(&app_state, &mut ctx, req, tmpl, "calculations_form.html");
        }
        _ => (),
    }

    app_state.apply("export", &mut ctx);

    ctx.insert(
        "print_url",
        &format!(
            "/print?{}",
            req.query_string(),
        ),
    );

    let rendered = tmpl.render("export_form.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

