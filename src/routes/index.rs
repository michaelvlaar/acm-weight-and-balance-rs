use crate::models::query_params::{FuelOptionQueryParams, IndexQueryParams, PerfQueryParams, WindOptionQueryParams};
use crate::utils::{parser, plane};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use airplane::weight_and_balance::Mass;
use tera::Tera;

use super::performance;

pub async fn index(
    query: web::Query<IndexQueryParams>,
    req: HttpRequest,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let mut ctx = tera::Context::new();

    let headers = req.headers();

    let template = if headers.get("HX-Request").is_some() {
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
        ) = parser::parse_query(query);

        let plane = plane::build_plane(
            callsign.clone(),
            pilot,
            pilot_seat.clone(),
            passenger,
            passenger_seat.clone(),
            baggage,
            fuel_quantity,
            fuel_type.clone(),
            fuel_quantity_type.clone(),
            fuel_option.clone(),
        );

        let fuel_mass = plane
            .moments()
            .last()
            .expect("plane should have moments")
            .mass();

        let fuel_quantity = match fuel_type.as_str() {
            "avgas" => fuel_mass.to_avgas(),
            "mogas" => fuel_mass.to_mogas(),
            _ => panic!("invalid fuel type"),
        };

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

        let quantity = match fuel_quantity {
            Mass::Avgas(v) | Mass::Mogas(v) => match fuel_quantity_type.as_str() {
                "liter" => v.to_liter(),
                "gallon" => v.to_gallon(),
                _ => panic!("invalid volume"),
            },
            _ => panic!("invalid mass"),
        };

        ctx.insert(
            "fuel_quantity",
            &format!("{:.2}", quantity).parse::<f64>().unwrap(),
        );
        ctx.insert("fuel_type", &fuel_type);
        ctx.insert("fuel_quantity_type", &fuel_quantity_type);
        ctx.insert("fuel_option", "manual");

        ctx.insert(
            "wb_chart_image_url",
            &format!("/wb-chart?{}", req.query_string()),
        );

        ctx.insert("print_url", &format!("/print?{}", req.query_string()));
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
        let (_, _, _, _, _, _, _, lgrr, ldr) = performance::calculate_aquila_performance_ldr(PerfQueryParams {
            mtow: plane.total_mass().kilo(),
            wind_direction: wind_direction.clone(),
            wind,
            pressure_altitude,
            oat,
        });

        let (_, _, _, _, _, _, _, tod_gr, tod_dr) =
            performance::calculate_aquila_performance_tod(PerfQueryParams {
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

        ctx.insert("wb_table", &airplane::visualizer::weight_and_balance_table_strings(plane));
        ctx.insert("stepper_oob_swap", &true);
        ctx.insert("step2", &true);

        "performance_form.html"
    } else {
        ctx.insert("show_image", &false);
        ctx.insert("fuel_option", "auto");
        ctx.insert("skip_stepper", &true);
        "index.html"
    };

    let rendered = tmpl.render(template, &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
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

pub async fn fuel_option(
    query: web::Query<FuelOptionQueryParams>,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let mut ctx = tera::Context::new();
    let query_params = query.into_inner();

    ctx.insert(
        "fuel_option",
        &query_params
            .fuel_option
            .unwrap_or_else(|| "manual".to_string()),
    );
    if query_params.fuel_quantity.is_some() {
        let q: f64 = query_params
            .fuel_quantity
            .unwrap()
            .parse()
            .unwrap_or_default();
        ctx.insert("fuel_quantity", &q);
    }

    ctx.insert("fuel_type", &query_params.fuel_type.unwrap());
    ctx.insert(
        "fuel_quantity_type",
        &query_params.fuel_quantity_type.unwrap(),
    );

    let rendered = tmpl.render("wb_form_fuel_option.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}
