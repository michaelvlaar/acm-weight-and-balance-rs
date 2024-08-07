use actix_web::{web, HttpRequest, HttpResponse, Responder};
use airplane::visualizer::weight_and_balance_table_strings;
use tera::Tera;

use crate::{models::query_params::{IndexQueryParams, PerfQueryParams}, utils::{parser::parse_query, plane::build_plane}};

use super::performance::{calculate_aquila_performance_ldr, calculate_aquila_performance_tod};

pub async fn print(
    query: web::Query<IndexQueryParams>,
    req: HttpRequest,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let mut ctx = tera::Context::new();

    ctx.insert(
        "wb_chart_image_url",
        &format!("/wb-chart?{}", req.query_string()),
    );

    let document_reference = query.reference.clone();

    let (
        callsign,
        pilot,
        pilot_seat,
        passenger,
        passenger_seat,
        baggage,
        fuel_quantity,
        fuel_type,
        fuel_quantity_type,
        fuel_option,
        oat,
        pressure_altitude,
        wind,
        wind_direction,
        _,
    ) = parse_query(query);

    let plane = build_plane(
        callsign.clone(),
        pilot,
        pilot_seat,
        passenger,
        passenger_seat,
        baggage,
        fuel_quantity,
        fuel_type.clone(),
        fuel_quantity_type.clone(),
        fuel_option.clone(),
    );

    ctx.insert(
        "perf_chart_tod_image_url",
        &format!(
            "/perf-tod?{}&mtow={}",
            req.query_string(),
            &plane.total_mass().kilo()
        ),
    );
    ctx.insert(
        "perf_chart_ldr_image_url",
        &format!(
            "/perf-ldr?{}&mtow={}",
            req.query_string(),
            &plane.total_mass().kilo()
        ),
    );

    let (_, _, _, _, _, _, _, lgrr, ldr) = calculate_aquila_performance_ldr(PerfQueryParams {
        mtow: plane.total_mass().kilo(),
        wind_direction: wind_direction.clone(),
        wind,
        pressure_altitude,
        oat,
    });

    let (_, _, _, _, _, _, _, tod_gr, tod_dr) = calculate_aquila_performance_tod(PerfQueryParams {
        mtow: plane.total_mass().kilo(),
        wind_direction,
        wind,
        pressure_altitude,
        oat,
    });

    ctx.insert("ldr", &format!("{:.0}", ldr));
    ctx.insert("lgrr", &format!("{:.0}", lgrr));
    ctx.insert("torr", &format!("{:.0}", tod_gr));
    ctx.insert("todr", &format!("{:.0}", tod_dr));

    ctx.insert("wb_table", &weight_and_balance_table_strings(plane));
    ctx.insert(
        "document_reference",
        &document_reference.unwrap_or_default(),
    );
    ctx.insert("print", &true);

    let rendered = tmpl.render("print.html", &ctx).unwrap();

    HttpResponse::Ok().content_type("text/html").body(rendered)
}
