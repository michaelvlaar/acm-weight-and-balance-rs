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
}

#[derive(Deserialize)]
struct FuelOptionQueryParams {
    fuel_option: Option<String>,
    fuel_quantity: Option<String>,
    fuel_type: Option<String>,
    fuel_quantity_type: Option<String>,
}

#[derive(RustEmbed)]
#[folder = "templates/"]
struct Templates;

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
) -> (String, f64, f64, f64, f64, String, String, String) {
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

    (
        callsign,
        pilot,
        passenger,
        baggage,
        fuel_quantity,
        fuel_type,
        fuel_quantity_type,
        fuel_option,
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
            .route("/print", web::get().to(print))
    })
    .bind("0.0.0.0:80")?
    .run()
    .await
}
