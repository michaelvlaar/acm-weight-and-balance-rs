use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use airplane::{
    visualizer::{
        weight_and_balance_table_strings, WeightBalanceChartVisualization,
        WeightBalanceTableVisualization,
    },
    weight_and_balance::{Airplane, CenterOfGravity, LeverArm, Limits, Mass, Moment, Volume},
};
use rust_embed::RustEmbed;
use serde::Deserialize;
use tera::Tera;

#[derive(Deserialize)]
struct IndexQueryParams {
    callsign: Option<String>,
    pilot: Option<f64>,
    passenger: Option<String>,
    bagage: Option<String>,
    fuel_option: Option<String>,
    fuel_quantity: Option<String>,
    fuel_type: Option<String>,
    fuel_quantity_type: Option<String>,
    reference: Option<String>,
    oat: Option<f64>,
    pressure_altitude: Option<f64>,
    wind: Option<f64>,
    wind_direction: Option<String>,
}

#[derive(Deserialize)]
struct PerfQueryParams {
    oat: f64,
    pressure_altitude: f64,
    mtow: f64,
    wind: f64,
    wind_direction: String,
}

#[derive(Deserialize)]
struct FuelOptionQueryParams {
    fuel_option: Option<String>,
    fuel_quantity: Option<String>,
    fuel_type: Option<String>,
    fuel_quantity_type: Option<String>,
}

#[derive(Deserialize)]
struct WindOptionQueryParams {
    wind: Option<f64>,
    wind_direction: Option<String>,
}

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

fn calculate_aquila_performance_tod(
    query_params: PerfQueryParams,
) -> (f64, f64, f64, f64, f64, f64, f64, f64, f64) {
    let oat = query_params.oat;
    let pressure_altitude = query_params.pressure_altitude;
    let mtow = query_params.mtow;
    let wind = query_params.wind;
    let wind_direction = query_params.wind_direction;

    let wind = if wind_direction == "headwind" { wind } else { -wind };

    let oat_x_start = 562.923177;
    let oat_x_end = 2168.91276;
    let oat_x_units = 70.0;

    let oat_y = [
        (
            0.0,
            [
                1614.322917,
                1656.315104,
                1698.339844,
                1742.317708,
                1788.313802,
                1834.342448,
                1882.324219,
                1932.324219,
            ],
        ),
        (
            2000.0,
            [
                1702.34375,
                1750.325521,
                1800.325521,
                1850.325521,
                1902.34375,
                1956.315104,
                2010.31901,
                2066.341146,
            ],
        ),
        (
            4000.0,
            [
                1804.329427,
                1860.31901,
                1916.341146,
                1974.316406,
                2034.342448,
                2096.321615,
                2160.31901,
                2224.316406,
            ],
        ),
        (
            6000.0,
            [
                1924.316406,
                1988.313802,
                2052.34375,
                2120.345052,
                2190.332031,
                2262.33724,
                2334.342448,
                2410.31901,
            ],
        ),
        (
            8000.0,
            [
                2064.322917,
                2138.313802,
                2214.322917,
                2292.317708,
                2372.330729,
                2456.315104,
                2540.332031,
                2628.320313,
            ],
        ),
    ];

    let y_bracket = ((oat + 30.0) / 10.0) as usize;
    let y_interpolated = interpolate_y_values(pressure_altitude, &oat_y, y_bracket);

    let p_oat_x = (oat_x_end - oat_x_start) / oat_x_units;
    let y_offset = (oat + 30.0) % 10.0;
    let p_oat_y = (y_interpolated.1 - y_interpolated.0) / 10.0;

    let tom_x_start = 2367.122396;
    let tom_x_end = 3777.246094;
    let tom_units = 750.0 - 550.0;
    let tom_x_offset = (750.0 - mtow) * ((tom_x_end - tom_x_start) / tom_units) + tom_x_start;

    let tom_y_pos = interpolate_tom_y(mtow, y_interpolated, p_oat_y, y_offset);
    let (wind_x_pos, wind_y_pos) = calculate_wind_position(wind, tom_y_pos, tom_x_offset);

    let obs = ((1395.703125, 1727.766927), (1491.731771, 1905.794271));
    let obs_y_pos = interpolate_obstacle_y(wind_y_pos, obs);

    let perf_y_start = 1009.635417;
    let perf_y_end = 4222.200521;
    let perf_units = 1000.0;

    let tor_gr = [
        1395.703125,
        1491.731771,
        1587.727865,
        1683.75651,
        1779.785156,
        1877.799479,
        1973.795573,
        2069.824219,
        2165.852865,
        2261.848958,
        2359.895833,
        2455.891927,
        2551.920573,
        2655.924479,
    ]
    .iter()
    .find(|&&x| x >= wind_y_pos)
    .unwrap_or(&perf_y_end);

    let tor_dr = [
        1727.766927,
        1905.794271,
        2085.839844,
        2265.852865,
        2443.880208,
        2623.925781,
        2803.938802,
        2983.984375,
        3162.011719,
        3342.057292,
        3522.070313,
        3700.097656,
        3880.143229,
        4076.171875,
    ]
    .iter()
    .find(|&&x| x >= obs_y_pos)
    .unwrap_or(&perf_y_end);

    let tod_gr = (tor_gr - perf_y_start) / (perf_y_end - perf_y_start) * perf_units;
    let tod_dr = (tor_dr - perf_y_start) / (perf_y_end - perf_y_start) * perf_units;

    (
        oat_x_start + (p_oat_x * (oat + 30.0)),
        y_interpolated.0 + (p_oat_y * y_offset),
        tom_x_offset,
        tom_y_pos,
        wind_x_pos,
        wind_y_pos,
        obs_y_pos,
        tod_gr,
        tod_dr,
    )
}

fn interpolate_y_values(
    pressure_altitude: f64,
    oat_y: &[(f64, [f64; 8])],
    y_bracket: usize,
) -> (f64, f64) {
    let y = if pressure_altitude <= 2000.0 {
        (oat_y[0], oat_y[1])
    } else if pressure_altitude <= 4000.0 {
        (oat_y[1], oat_y[2])
    } else if pressure_altitude <= 6000.0 {
        (oat_y[2], oat_y[3])
    } else if pressure_altitude <= 8000.0 {
        (oat_y[3], oat_y[4])
    } else {
        panic!("not within range");
    };

    let y_factor = (pressure_altitude - y.0 .0) / (y.1 .0 - y.0 .0);
    (
        interpolate(y.0 .1[y_bracket], y.1 .1[y_bracket], y_factor),
        interpolate(y.0 .1[if y_bracket + 1 < y.0.1.len() { y_bracket + 1} else { y_bracket }], y.1 .1[if y_bracket + 1 < y.1.1.len() { y_bracket + 1} else { y_bracket }], y_factor),
    )
}

fn interpolate(start: f64, end: f64, factor: f64) -> f64 {
    start + (end - start) * factor
}

fn interpolate_tom_y(mtow: f64, y_interpolated: (f64, f64), p_oat_y: f64, y_offset: f64) -> f64 {
    let tom = (
        (0.0, 200.0, 1632.03125, 1400.032552),
        (0.0, 200.0, 1718.033854, 1454.003906),
    );
    let tom_y = (
        interpolate(tom.0 .2, tom.0 .3, (750.0 - mtow) / (tom.0 .1 - tom.0 .0)),
        interpolate(tom.1 .2, tom.1 .3, (750.0 - mtow) / (tom.1 .1 - tom.1 .0)),
    );
    interpolate(
        tom_y.0,
        tom_y.1,
        (y_interpolated.0 + (p_oat_y * y_offset) - tom.0 .2) / (tom.1 .2 - tom.0 .2),
    )
}

fn calculate_wind_position(wind: f64, tom_y_pos: f64, tom_x_offset: f64) -> (f64, f64) {
    let mut wind_x_pos = tom_x_offset;
    let mut wind_y_pos = tom_y_pos;

    if wind != 0.0 {
        let wind_x_start = 3965.429687;
        let wind_x_end = 5211.621094;
        let wind_units = 20.0;
        let wind_x_offset = wind.abs() * ((wind_x_end - wind_x_start) / wind_units) + wind_x_start;
        let mut wind_offset = wind;

        let initial_factor = (
            (0.0, 10.0, 1389.84375, 1303.841146),
            (0.0, 10.0, 1655.891927, 1507.877604),
        );

        let wind_d = if (0.0..=10.0).contains(&wind) {
            initial_factor
        } else if wind > 10.0 && wind <= 15.0 {
            wind_offset = wind % 10.0;
            (
                (0.0, 5.0, 1303.841146, 1269.856771),
                (0.0, 5.0, 1507.877604, 1449.869792),
            )
        } else if (-10.0..0.0).contains(&wind) {
            if tom_y_pos <= 1640.891927 {
                (
                    (0.0, 10.0, 1389.84375, 1525.84375),
                    (0.0, 10.0, 1640.891927, 1867.0),
                )
            } else if tom_y_pos <= 1958.915365 {
                (
                    (0.0, 10.0, 1640.891927, 1867.0),
                    (0.0, 10.0, 1958.915365, 2300.0),
                )
            } else {
                (
                    (0.0, 10.0, 1958.915365, 2300.0),
                    (0.0, 10.0, 2262.979167, 2710.0),
                )
            }
        } else {
            wind_offset = wind % 15.0;
            (
                (0.0, 5.0, 1269.856771, 1243.847656),
                (0.0, 5.0, 1449.869792, 1407.845052),
            )
        };

        let wind_low = wind_d.0 .2
            - ((wind_d.0 .2 - wind_d.0 .3) / (wind_d.0 .1 - wind_d.0 .0)) * wind_offset.abs();
        let wind_high = wind_d.1 .2
            - ((wind_d.1 .2 - wind_d.1 .3) / (wind_d.1 .1 - wind_d.1 .0)) * wind_offset.abs();

        let wind_factor = if wind >= 0.0 {
            (tom_y_pos - initial_factor.0 .2) / (initial_factor.1 .2 - initial_factor.0 .2)
        } else {
            (tom_y_pos - wind_d.0 .2) / (wind_d.1 .2 - wind_d.0 .2)
        };

        wind_y_pos = (wind_high - wind_low) * wind_factor + wind_low;
        wind_x_pos = wind_x_offset.abs();
    }

    (wind_x_pos, wind_y_pos)
}

fn interpolate_obstacle_y(wind_y_pos: f64, obs: ((f64, f64), (f64, f64))) -> f64 {
    let obs_factor = (wind_y_pos - obs.0 .0) / (obs.1 .0 - obs.0 .0);
    obs.0 .1 + (obs_factor) * (obs.1 .1 - obs.0 .1)
}

async fn perf_tod(
    query: web::Query<PerfQueryParams>,
    _req: HttpRequest,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let mut ctx = tera::Context::new();

    let (oat_x_base, oat_y_base, tom_x_offset, tom_y_pos, wind_x_pos, wind_y_pos, obs_y_pos, _, _) =
        calculate_aquila_performance_tod(query.into_inner());

    ctx.insert("oat_x_base", &format!("{:.5}", oat_x_base));
    ctx.insert("oat_y_base", &format!("{:.5}", oat_y_base));
    ctx.insert("tom_x", &format!("{:.5}", tom_x_offset));
    ctx.insert("tom_y", &format!("{:.5}", tom_y_pos,));
    ctx.insert("wind_x", &format!("{:.5}", wind_x_pos));
    ctx.insert("wind_y", &format!("{:.5}", wind_y_pos));
    ctx.insert("ob_y", &format!("{:.5}", obs_y_pos));

    let rendered = tmpl.render("top.svg", &ctx).unwrap();
    HttpResponse::Ok()
        .content_type("image/svg+xml")
        .body(rendered)
}

async fn print(
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
        passenger,
        baggage,
        fuel_quantity,
        fuel_type,
        fuel_quantity_type,
        fuel_option,
        _,
        _,
        _,
        _,
    ) = parse_query(query);

    let plane = build_plane(
        callsign.clone(),
        pilot,
        passenger,
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
    ctx.insert("wb_table", &weight_and_balance_table_strings(plane));
    ctx.insert(
        "document_reference",
        &document_reference.unwrap_or_default(),
    );
    ctx.insert("print", &true);

    let rendered = tmpl.render("print.html", &ctx).unwrap();

    HttpResponse::Ok().content_type("text/html").body(rendered)
}

async fn index(
    query: web::Query<IndexQueryParams>,
    req: HttpRequest,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let mut ctx = tera::Context::new();

    let headers = req.headers();

    let template = if headers.get("HX-Request").is_some() {
        ctx.insert("show_image", &true);


        let (
            callsign,
            pilot,
            passenger,
            baggage,
            fuel_quantity,
            fuel_type,
            fuel_quantity_type,
            fuel_option,
            oat,
            pressure_altitude,
            wind,
            wind_direction,
        ) = parse_query(query);

        let plane = build_plane(
            callsign.clone(),
            pilot,
            passenger,
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
        ctx.insert("passenger", &passenger);
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
        ctx.insert("wb_table", &weight_and_balance_table_strings(plane));

        "wb_form.html"
    } else {
        ctx.insert("show_image", &false);
        ctx.insert("fuel_option", "auto");
        "index.html"
    };

    let rendered = tmpl.render(template, &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

async fn wind_option(
    query: web::Query<WindOptionQueryParams>,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let mut ctx = tera::Context::new();
    let query_params = query.into_inner();

    if query_params.wind.is_some() {
        ctx.insert("wind", &query_params.wind.unwrap());
    }

    ctx.insert("wind_direction", &query_params.wind_direction.unwrap_or_else(|| "headwind".to_string()));

    let rendered = tmpl.render("wb_form_wind_option.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}
async fn fuel_option(
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

fn build_plane(
    callsign: String,
    pilot: f64,
    passenger: f64,
    baggage: f64,
    fuel_quantity: f64,
    fuel_type: String,
    fuel_quantity_type: String,
    fuel_option: String,
) -> Airplane {
    let empty_mass = if callsign == "PHDHA" { 517.0 } else { 529.5 };

    let mut plane = Airplane::new(
        callsign,
        vec![
            Moment::new(
                "Empty Mass".to_string(),
                LeverArm::Meter(0.4294),
                Mass::Kilo(empty_mass),
            ),
            Moment::new(
                "Pilot".to_string(),
                LeverArm::Meter(0.515),
                Mass::Kilo(pilot),
            ),
            Moment::new(
                "Passenger".to_string(),
                LeverArm::Meter(0.515),
                Mass::Kilo(passenger),
            ),
            Moment::new(
                "Baggage".to_string(),
                LeverArm::Meter(1.3),
                Mass::Kilo(baggage),
            ),
        ],
        Limits::new(
            Mass::Kilo(558.0),
            Mass::Kilo(750.0),
            CenterOfGravity::Millimeter(427.0),
            CenterOfGravity::Millimeter(523.0),
        ),
    );

    if fuel_option == "auto" {
        plane.add_max_mass_within_limits(
            "Fuel".to_string(),
            LeverArm::Meter(0.325),
            match fuel_type.as_str() {
                "mogas" => Mass::Mogas(match fuel_quantity_type.as_str() {
                    "liter" => Volume::Liter(0.0),
                    "gallon" => Volume::Gallon(0.0),
                    _ => panic!("invalid volume type"),
                }),
                "avgas" => Mass::Avgas(match fuel_quantity_type.as_str() {
                    "liter" => Volume::Liter(0.0),
                    "gallon" => Volume::Gallon(0.0),
                    _ => panic!("invalid volume type"),
                }),
                _ => panic!("invalid fuel type"),
            },
            Some(Volume::Liter(110.0)),
        );
    } else {
        let fuel_volume = match fuel_quantity_type.as_str() {
            "liter" => Volume::Liter(fuel_quantity),
            "gallon" => Volume::Gallon(fuel_quantity),
            _ => panic!("invalid volume type"),
        };

        let fuel_mass = match fuel_type.as_str() {
            "mogas" => Mass::Mogas(fuel_volume),
            "avgas" => Mass::Avgas(fuel_volume),
            _ => panic!("invalid fuel type"),
        };

        plane.add_moment(Moment::new(
            "Fuel".to_string(),
            LeverArm::Meter(0.325),
            fuel_mass,
        ));
    }

    plane
}

fn parse_query(
    query: web::Query<IndexQueryParams>,
) -> (
    String,
    f64,
    f64,
    f64,
    f64,
    String,
    String,
    String,
    f64,
    f64,
    f64,
    String,
) {
    let query_params = query.into_inner();
    let callsign = query_params.callsign.expect("calsign must be present.");
    let pilot = query_params.pilot.expect("pilot should be present.");
    let passenger: f64 = query_params
        .passenger
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();
    let baggage: f64 = query_params
        .bagage
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();
    let fuel_quantity: f64 = query_params
        .fuel_quantity
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();
    let fuel_type = query_params
        .fuel_type
        .expect("fuel type should be present.");
    let fuel_quantity_type = query_params
        .fuel_quantity_type
        .expect("fuel quantity type should be present.");
    let fuel_option = query_params
        .fuel_option
        .unwrap_or_else(|| "manual".to_string());
    let oat = query_params
        .oat
        .expect("outside air temperature should be present.");
    let pressure_altitude = query_params
        .pressure_altitude
        .expect("pressure altitude should be present.");
    let wind = query_params.wind.expect("wind should be present.");
    let wind_direction = query_params.wind_direction.unwrap_or_else(|| "headwind".to_string());
    (
        callsign,
        pilot,
        passenger,
        baggage,
        fuel_quantity,
        fuel_type,
        fuel_quantity_type,
        fuel_option,
        oat,
        pressure_altitude,
        wind,
        wind_direction,
    )
}

async fn wb_table(query: web::Query<IndexQueryParams>, _tmpl: web::Data<Tera>) -> impl Responder {
    let mut ctx = tera::Context::new();
    ctx.insert("show_image", &true);

    let (
        callsign,
        pilot,
        passenger,
        baggage,
        fuel_quantity,
        fuel_type,
        fuel_quantity_type,
        fuel_option,
        _,
        _,
        _,
        _,
    ) = parse_query(query);

    let plane = build_plane(
        callsign,
        pilot,
        passenger,
        baggage,
        fuel_quantity,
        fuel_type,
        fuel_quantity_type,
        fuel_option,
    );

    match airplane::visualizer::weight_and_balance_table(
        plane,
        WeightBalanceTableVisualization::new((620, 220)),
    ) {
        airplane::visualizer::Visualization::Svg(svg) => {
            return HttpResponse::Ok().content_type("image/svg+xml").body(svg);
        }
    };
}

async fn wb_chart(query: web::Query<IndexQueryParams>, _tmpl: web::Data<Tera>) -> impl Responder {
    let mut ctx = tera::Context::new();
    ctx.insert("show_image", &true);

    let (
        callsign,
        pilot,
        passenger,
        baggage,
        fuel_quantity,
        fuel_type,
        fuel_quantity_type,
        fuel_option,
        _,
        _,
        _,
        _,
    ) = parse_query(query);

    let plane = build_plane(
        callsign,
        pilot,
        passenger,
        baggage,
        fuel_quantity,
        fuel_type,
        fuel_quantity_type,
        fuel_option,
    );

    match airplane::visualizer::weight_and_balance_chart(
        plane,
        WeightBalanceChartVisualization::new((500, 500), (230.0..420.0, 550.0..760.0)),
    ) {
        airplane::visualizer::Visualization::Svg(svg) => {
            return HttpResponse::Ok().content_type("image/svg+xml").body(svg);
        }
    };
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut tera = Tera::default();
    for file in Templates::iter() {
        if let Some(content) = Templates::get(file.as_ref()) {
            let content_str = std::str::from_utf8(&content.data).unwrap();
            tera.add_raw_template(file.as_ref(), content_str).unwrap();
        }
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .route("/", web::get().to(index))
            .route("/wb-chart", web::get().to(wb_chart))
            .route("/wb-table", web::get().to(wb_table))
            .route("/fuel-option", web::get().to(fuel_option))
            .route("/wind-option", web::get().to(wind_option))
            .route("/print", web::get().to(print))
            .route("/perf-tod", web::get().to(perf_tod))
    })
    .bind("0.0.0.0:80")?
    .run()
    .await
}
