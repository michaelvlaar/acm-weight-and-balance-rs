use actix_web::{web, HttpRequest, HttpResponse, Responder};
use airplane::visualizer::weight_and_balance_table_strings;
use tera::Tera;

use crate::{models::query_params::{IndexQueryParams, PerfQueryParams}, utils::{parser::parse_query, plane::build_plane}};

use super::performance::{calculate_aquila_performance_ldr, calculate_aquila_performance_tod};

pub async fn export(
    query: web::Query<IndexQueryParams>,
    req: HttpRequest,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let mut ctx = tera::Context::new();

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
        submit,
    ) = parse_query(query);

    ctx.insert("callsign", &callsign);
    ctx.insert("pilot", &pilot);
    ctx.insert("pilot_seat", &pilot_seat);
    ctx.insert("passenger", &passenger);
    ctx.insert("passenger_seat", &passenger_seat);
    ctx.insert("baggage", &baggage);
    ctx.insert("oat", &oat);
    ctx.insert("pressure_altitude", &pressure_altitude);
    ctx.insert("wind", &wind);
    ctx.insert("wind_direction", &wind_direction);
    ctx.insert(
        "fuel_quantity",
        &format!("{:.2}", &fuel_quantity).parse::<f64>().unwrap(),
    );
    ctx.insert("fuel_type", &fuel_type);
    ctx.insert("fuel_quantity_type", &fuel_quantity_type);
    ctx.insert("fuel_option", "manual");
    ctx.insert("stepper_oob_swap", &true);

    if submit.eq("Vorige") {
        ctx.insert("step2", &true);

        ctx.insert(
            "wb_chart_image_url",
            &format!("/wb-chart?{}", req.query_string()),
        );
    
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

        let (_, _, _, _, _, _, _, lgrr, ldr) = calculate_aquila_performance_ldr(PerfQueryParams{
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

        let rendered = tmpl.render("performance_form.html", &ctx).unwrap();
        return HttpResponse::Ok().content_type("text/html").body(rendered);
    }

    let rendered = tmpl.render("export_form.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

