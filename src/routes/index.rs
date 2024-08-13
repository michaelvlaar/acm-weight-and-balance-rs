use crate::models::query_params::{IndexQueryParams, WindOptionQueryParams};
use crate::models::state::ApplicationState;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use tera::Tera;

pub async fn index(
    query: web::Query<IndexQueryParams>,
    req: HttpRequest,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let mut ctx = tera::Context::new();

    let headers = req.headers();

    let (app_state, _) = ApplicationState::from_query_params(query.into_inner());
    let mut step = "input";

    let template = if headers.get("HX-Request").is_some() {
        step = "fuel";
        "fuel_form.html"
    } else {
        "index.html"
    };

    app_state.apply(step, &mut ctx);

    let rendered = tmpl.render(template, &ctx).unwrap();
    HttpResponse::Ok()
        .insert_header(("Link", "<https://unpkg.com/htmx.org@2.0.1>; rel=preload; as=script"))
        .insert_header(("Link", "<https://cdn.jsdelivr.net/npm/tailwindcss@2.2.19/dist/tailwind.min.css>; rel=preload; as=style"))
        .insert_header(("Link", "<https://cdn.jsdelivr.net/npm/flatpickr/dist/flatpickr.min.css>; rel=preload; as=style"))
        .insert_header(("Link", "<https://fonts.googleapis.com/icon?family=Material+Icons>; rel=preload; as=style"))
        .insert_header(("Link", "<https://cdn.jsdelivr.net/npm/flatpickr>; rel=preload; as=script"))
        .insert_header(("Link", "<data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%221em%22 font-size=%2280%22>&#128747;</text></svg>>; rel=preload; as=image"))        .content_type("text/html")
        .body(rendered)
}

pub async fn wind_option(
    query: web::Query<WindOptionQueryParams>,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let mut ctx = tera::Context::new();
    let query_params = query.into_inner();

    if query_params.wind.is_some() {
        ctx.insert("wind", &query_params.wind.unwrap());
    }

    ctx.insert(
        "wind_direction",
        &query_params
            .wind_direction
            .unwrap_or_else(|| "headwind".to_string()),
    );

    let rendered = tmpl.render("wb_form_wind_option.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}
