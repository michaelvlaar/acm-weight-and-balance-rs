use core::panic;
use std::time::Duration;

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use airplane::{
    types::VolumeType,
    visualizer::{WeightBalanceChartVisualization, WeightBalanceTableVisualization},
    weight_and_balance::Volume,
};
use tera::Tera;

use crate::{
    models::{
        query_params::{IndexQueryParams, PerfQueryParams},
        state::{duration_to_hh_mm, ApplicationState},
    },
    utils::plane,
};

pub async fn calculations(
    query: web::Query<IndexQueryParams>,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let mut ctx = tera::Context::new();
    let (app_state, query) = ApplicationState::from_query_params(query.into_inner());

    match query.submit {
        Some(s) if s == "Vorige" => {
            app_state.apply("fuel", &mut ctx);
            let rendered = tmpl.render("fuel_form.html", &ctx).unwrap();
            return HttpResponse::Ok().content_type("text/html").body(rendered);
        }
        _ => (),
    }

    let rendered = tmpl.render("export_form.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

pub fn render_calculations(
    app_state: &ApplicationState,
    ctx: &mut tera::Context,
    req: HttpRequest,
    tmpl: web::Data<Tera>,
    template: &str,
) -> HttpResponse {
    let plane = plane::build_plane(
        app_state.callsign.clone().unwrap(),
        app_state.pilot_moment.clone().unwrap(),
        app_state.passenger_moment.clone(),
        app_state.baggage_moment.clone(),
        app_state.fuel_type.clone().unwrap(),
        app_state.fuel_unit.clone().unwrap(),
        app_state.fuel_extra.clone(),
        app_state.fuel_max.unwrap_or_default(),
        app_state.trip_duration.unwrap(),
    );

    app_state.apply("calculation", ctx);

    if let Some(fuel_moment) = plane.moments().last() {
        let fuel_mass = fuel_moment.mass();

        let fuel_liters = match fuel_mass {
            airplane::weight_and_balance::Mass::Mogas(v)
            | airplane::weight_and_balance::Mass::Avgas(v) => v.to_liter(),
            _ => panic!("should be a fuel"),
        };

        let taxi = match app_state.fuel_unit {
            Some(VolumeType::Liter) => Volume::Liter(2.0),
            Some(VolumeType::Gallon) => Volume::Gallon(Volume::Liter(2.0).to_gallon()),
            None => panic!("should never be none"),
        };

        let reserve = match app_state.fuel_unit {
            Some(VolumeType::Liter) => Volume::Liter(17.0 * 0.75),
            Some(VolumeType::Gallon) => Volume::Gallon(Volume::Liter(17.0 * 0.75).to_gallon()),
            None => panic!("should never be none"),
        };

        let trip = match app_state.fuel_unit {
            Some(VolumeType::Liter) => Volume::Liter(
                17.0 * (app_state
                    .trip_duration
                    .expect("should have duration")
                    .as_secs_f64()
                    / 60.0
                    / 60.0),
            ),
            Some(VolumeType::Gallon) => Volume::Gallon(
                Volume::Liter(
                    17.0 * (app_state
                        .trip_duration
                        .expect("should have duration")
                        .as_secs_f64()
                        / 60.0
                        / 60.0),
                )
                .to_gallon(),
            ),
            None => panic!("should never be none"),
        };

        let alternate = match app_state.fuel_unit {
            Some(VolumeType::Liter) => Volume::Liter(
                17.0 * (app_state
                    .alternate_duration
                    .expect("should have duration")
                    .as_secs_f64()
                    / 60.0
                    / 60.0),
            ),
            Some(VolumeType::Gallon) => Volume::Gallon(
                Volume::Liter(
                    17.0 * (app_state
                        .alternate_duration
                        .expect("should have duration")
                        .as_secs_f64()
                        / 60.0
                        / 60.0),
                )
                .to_gallon(),
            ),
            None => panic!("should never be none"),
        };

        let contigency = match app_state.fuel_unit {
            Some(VolumeType::Liter) => Volume::Liter(trip.to_liter() * 0.1),
            Some(VolumeType::Gallon) => {
                Volume::Gallon(Volume::Liter(trip.to_liter() * 0.1).to_gallon())
            }
            None => panic!("should never be none"),
        };

        let extra = match app_state.fuel_unit {
            Some(VolumeType::Liter) => Volume::Liter(
                fuel_liters
                    - taxi.to_liter()
                    - reserve.to_liter()
                    - trip.to_liter()
                    - alternate.to_liter()
                    - contigency.to_liter(),
            ),
            Some(VolumeType::Gallon) => Volume::Gallon(
                Volume::Liter(
                    fuel_liters
                        - taxi.to_liter()
                        - reserve.to_liter()
                        - trip.to_liter()
                        - alternate.to_liter()
                        - contigency.to_liter(),
                )
                .to_gallon(),
            ),
            None => panic!("should never be none"),
        };

        let endurance = Duration::from_secs((fuel_liters / 17.0 * 60.0 * 60.0) as u64);

        ctx.insert("fuel_taxi", &taxi.to_string().replace('.', ","));
        ctx.insert("fuel_reserve", &reserve.to_string().replace('.', ","));
        ctx.insert("fuel_trip", &trip.to_string().replace('.', ","));
        ctx.insert("fuel_alternate", &alternate.to_string().replace('.', ","));
        ctx.insert("fuel_contigency", &contigency.to_string().replace('.', ","));
        ctx.insert("fuel_additional", &extra.to_string().replace('.', ","));
        ctx.insert(
            "fuel_additional_abs",
            &match extra {
                Volume::Gallon(v) => Volume::Gallon(v.abs()),
                Volume::Liter(v) => Volume::Liter(v.abs()),
            }
            .to_string()
            .replace('.', ","),
        );
        ctx.insert(
            "fuel_sufficient",
            &match extra {
                Volume::Gallon(v) | Volume::Liter(v) => v.is_sign_positive(),
            },
        );

        ctx.insert(
            "fuel_total",
            &match app_state.fuel_unit {
                Some(VolumeType::Liter) => Volume::Liter(fuel_liters),
                Some(VolumeType::Gallon) => Volume::Gallon(Volume::Liter(fuel_liters).to_gallon()),
                _ => panic!("should always have a value"),
            }
            .to_string()
            .replace('.', ","),
        );

        ctx.insert("fuel_endurance", &duration_to_hh_mm(&endurance));
    }

    ctx.insert("wb_within_limits", &plane.within_limits());

    let wind = match app_state.wind {
        Some(w) => w.abs(),
        None => panic!("wind should be present"),
    };

    let wind_direction = match app_state.wind {
        Some(w) => {
            if w.is_sign_negative() {
                "tailwind".to_string()
            } else {
                "headwind".to_string()
            }
        }
        None => panic!("wind should be present"),
    };

    let pressure_altitude = match app_state.pressure_altitude {
        Some(pa) => pa,
        None => panic!("pressure altitude should be present"),
    };

    let oat = match app_state.oat {
        Some(oat) => oat,
        None => panic!("oat should be present"),
    };

    let (_, _, _, _, _, _, _, lgrr, ldr) = calculate_aquila_performance_ldr(PerfQueryParams {
        mtow: plane.total_mass_landing().kilo(),
        wind,
        wind_direction: wind_direction.clone(),
        pressure_altitude,
        oat,
    });

    let (_, _, _, _, _, _, _, tod_gr, tod_dr) = calculate_aquila_performance_tod(PerfQueryParams {
        mtow: plane.total_mass().kilo(),
        wind,
        wind_direction,
        pressure_altitude,
        oat,
    });

    ctx.insert("ldr", &format!("{:.0}", ldr));
    ctx.insert("lgrr", &format!("{:.0}", lgrr));
    ctx.insert("torr", &format!("{:.0}", tod_gr));
    ctx.insert("todr", &format!("{:.0}", tod_dr));

    ctx.insert(
        "perf_chart_tod_image_url",
        &format!(
            "/perf-tod?{}&mtow={}",
            req.query_string(),
            &plane.total_mass().kilo()
        ),
    );

    ctx.insert(
        "wb_chart_image_url",
        &format!("/wb-chart?{}", req.query_string()),
    );

    ctx.insert(
        "perf_chart_ldr_image_url",
        &format!(
            "/perf-ldr?{}&mtow={}",
            req.query_string(),
            &plane.total_mass_landing().kilo()
        ),
    );

    ctx.insert(
        "wb_table",
        &airplane::visualizer::weight_and_balance_table_strings(plane),
    );

    let rendered = tmpl.render(template, ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

pub fn calculate_aquila_performance_ldr(
    query_params: PerfQueryParams,
) -> (f64, f64, f64, f64, f64, f64, f64, f64, f64) {
    let oat = query_params.oat;
    let pressure_altitude = query_params.pressure_altitude;
    let mtow = query_params.mtow;
    let wind = query_params.wind;
    let wind_direction = query_params.wind_direction;

    let wind = if wind_direction == "headwind" {
        wind
    } else {
        -wind
    };

    let oat_x_start = 562.923177;
    let oat_x_end = 1870.93099;
    let oat_x_units = 70.0;

    let oat_y = [
        (
            0.0,
            [
                1902.34375,
                1948.339844,
                1994.335938,
                2042.317708,
                2090.332031,
                2136.328125,
                2184.342448,
                2234.342448,
            ],
        ),
        (
            2000.0,
            [
                2002.34375,
                2054.329427,
                2104.329427,
                2158.333333,
                2210.31901,
                2262.33724,
                2316.341146,
                2370.345052,
            ],
        ),
        (
            4000.0,
            [
                2114.322917,
                2172.330729,
                2228.320313,
                2286.328125,
                2344.335938,
                2404.329427,
                2462.33724,
                2522.330729,
            ],
        ),
        (
            6000.0,
            [
                2242.317708,
                2304.329427,
                2368.326823,
                2432.324219,
                2498.339844,
                2562.33724,
                2628.320313,
                2694.335938,
            ],
        ),
        (
            8000.0,
            [
                2384.342448,
                2454.329427,
                2526.334635,
                2598.339844,
                2670.345052,
                2742.317708,
                2814.322917,
                2888.313802,
            ],
        ),
    ];

    let y_bracket = ((oat + 30.0) / 10.0) as usize;
    let y_interpolated = interpolate_y_values(pressure_altitude, &oat_y, y_bracket);

    let p_oat_x = (oat_x_end - oat_x_start) / oat_x_units;
    let y_offset = (oat + 30.0) % 10.0;
    let p_oat_y = (y_interpolated.1 - y_interpolated.0) / 10.0;

    let tom_x_start = 2077.115885;
    let tom_x_end = 3263.216146;
    let tom_units = 750.0 - 550.0;
    let tom_x_offset = (750.0 - mtow) * ((tom_x_end - tom_x_start) / tom_units) + tom_x_start;

    let tom = if y_interpolated.0 + (p_oat_y * y_offset) <= 2002.083333 {
        (
            (0.0, 200.0, 1906.054688, 1796.061198),
            (0.0, 200.0, 2002.083333, 1882.063802),
        )
    } else if y_interpolated.0 + (p_oat_y * y_offset) <= 2112.076823 {
        (
            (0.0, 200.0, 2002.083333, 1882.063802),
            (0.0, 200.0, 2112.076823, 1978.059896),
        )
    } else if y_interpolated.0 + (p_oat_y * y_offset) <= 2232.096354 {
        (
            (0.0, 200.0, 2112.076823, 1978.059896),
            (0.0, 200.0, 2232.096354, 2074.088542),
        )
    } else {
        (
            (0.0, 200.0, 2232.096354, 2074.088542),
            (0.0, 200.0, 2368.098958, 2192.089844),
        )
    };

    let tom_y_pos = interpolate_tom_y(tom, mtow, y_interpolated, p_oat_y, y_offset);

    let wind_x_start = 3439.388021;
    let wind_x_end = 4933.561198;
    let (wind_x_pos, wind_y_pos) =
        calculate_wind_position_ldr(wind_x_start, wind_x_end, wind, tom_y_pos, tom_x_offset);

    let obs = ((1467.545573, 1171.484375), (1631.608073, 1241.503906));
    let gr_y_pos = interpolate_obstacle_y(wind_y_pos, obs);

    let perf_y_start = 965.46224;
    let perf_y_end = 3261.946615;
    let perf_units = 1000.0;

    let ldr_gr = [
        1171.484375,
        1241.503906,
        1309.53776,
        1379.557292,
        1447.558594,
        1517.578125,
        1585.579427,
        1653.613281,
        1723.632813,
        1791.634115,
        1861.653646,
        1929.654948,
        1999.674479,
    ]
    .iter()
    .find(|&&x| x >= gr_y_pos)
    .unwrap_or(&perf_y_end);

    let ldr_dr = [
        1467.545573,
        1631.608073,
        1797.65625,
        1961.686198,
        2125.716146,
        2289.746094,
        2453.776042,
        2617.80599,
        2781.835938,
        2947.884115,
        3111.914063,
        3275.94401,
        3440.00651,
    ]
    .iter()
    .find(|&&x| x >= wind_y_pos)
    .unwrap_or(&perf_y_end);

    let ldr_gr = (ldr_gr - perf_y_start) / (perf_y_end - perf_y_start) * perf_units;
    let ldr_dr = (ldr_dr - perf_y_start) / (perf_y_end - perf_y_start) * perf_units;

    (
        oat_x_start + (p_oat_x * (oat + 30.0)),
        y_interpolated.0 + (p_oat_y * y_offset),
        tom_x_offset,
        tom_y_pos,
        wind_x_pos,
        wind_y_pos,
        gr_y_pos,
        ldr_gr,
        ldr_dr,
    )
}

pub fn calculate_aquila_performance_tod(
    query_params: PerfQueryParams,
) -> (f64, f64, f64, f64, f64, f64, f64, f64, f64) {
    let oat = query_params.oat;
    let pressure_altitude = query_params.pressure_altitude;
    let mtow = query_params.mtow;
    let wind = query_params.wind;
    let wind_direction = query_params.wind_direction;

    let wind = if wind_direction == "headwind" {
        wind
    } else {
        -wind
    };

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

    let tom = (
        (0.0, 200.0, 1632.03125, 1400.032552),
        (0.0, 200.0, 1718.033854, 1454.003906),
    );
    let tom_y_pos = interpolate_tom_y(tom, mtow, y_interpolated, p_oat_y, y_offset);
    let wind_x_start = 3965.429687;
    let wind_x_end = 5211.621094;
    let (wind_x_pos, wind_y_pos) =
        calculate_wind_position_tod(wind_x_start, wind_x_end, wind, tom_y_pos, tom_x_offset);

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
        interpolate(
            y.0 .1[if y_bracket + 1 < y.0 .1.len() {
                y_bracket + 1
            } else {
                y_bracket
            }],
            y.1 .1[if y_bracket + 1 < y.1 .1.len() {
                y_bracket + 1
            } else {
                y_bracket
            }],
            y_factor,
        ),
    )
}

fn interpolate(start: f64, end: f64, factor: f64) -> f64 {
    start + (end - start) * factor
}

fn interpolate_tom_y(
    tom: ((f64, f64, f64, f64), (f64, f64, f64, f64)),
    mtow: f64,
    y_interpolated: (f64, f64),
    p_oat_y: f64,
    y_offset: f64,
) -> f64 {
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

fn calculate_wind_position_ldr(
    wind_x_start: f64,
    wind_x_end: f64,
    wind: f64,
    tom_y_pos: f64,
    tom_x_offset: f64,
) -> (f64, f64) {
    let mut wind_x_pos = tom_x_offset;
    let mut wind_y_pos = tom_y_pos;

    if wind != 0.0 {
        let wind_units = 20.0;
        let wind_x_offset = wind.abs() * ((wind_x_end - wind_x_start) / wind_units) + wind_x_start;
        let mut wind_offset = wind;

        let initial_factor = (
            (0.0, 10.0, 1787.923177, 1599.902344),
            (0.0, 10.0, 2173.958333, 1897.916667),
        );

        let wind_d = if (0.0..=10.0).contains(&wind) {
            initial_factor
        } else if wind > 10.0 && wind <= 15.0 {
            wind_offset = wind % 10.0;
            (
                (0.0, 5.0, 1599.902344, 1527.864583),
                (0.0, 5.0, 1897.916667, 1791.894531),
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
                (0.0, 5.0, 1527.864583, 1471.875),
                (0.0, 5.0, 1791.894531, 1709.895833),
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

fn calculate_wind_position_tod(
    wind_x_start: f64,
    wind_x_end: f64,
    wind: f64,
    tom_y_pos: f64,
    tom_x_offset: f64,
) -> (f64, f64) {
    let mut wind_x_pos = tom_x_offset;
    let mut wind_y_pos = tom_y_pos;

    if wind != 0.0 {
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

pub async fn perf_ldr(
    query: web::Query<PerfQueryParams>,
    _req: HttpRequest,
    tmpl: web::Data<Tera>,
) -> impl Responder {
    let mut ctx = tera::Context::new();

    let (oat_x_base, oat_y_base, tom_x_offset, tom_y_pos, wind_x_pos, wind_y_pos, obs_y_pos, _, _) =
        calculate_aquila_performance_ldr(query.into_inner());

    ctx.insert("oat_x_base", &format!("{:.5}", oat_x_base));
    ctx.insert("oat_y_base", &format!("{:.5}", oat_y_base));
    ctx.insert("tom_x", &format!("{:.5}", tom_x_offset));
    ctx.insert("tom_y", &format!("{:.5}", tom_y_pos,));
    ctx.insert("wind_x", &format!("{:.5}", wind_x_pos));
    ctx.insert("wind_y", &format!("{:.5}", wind_y_pos));
    ctx.insert("ob_y", &format!("{:.5}", obs_y_pos));

    let rendered = tmpl.render("ld.svg", &ctx).unwrap();
    HttpResponse::Ok()
        .content_type("image/svg+xml")
        .body(rendered)
}

pub async fn perf_tod(query: web::Query<PerfQueryParams>, tmpl: web::Data<Tera>) -> impl Responder {
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

pub async fn wb_table(
    query: web::Query<IndexQueryParams>,
    _tmpl: web::Data<Tera>,
) -> impl Responder {
    let (app_state, _) = ApplicationState::from_query_params(query.into_inner());

    let plane = plane::build_plane(
        app_state.callsign.unwrap(),
        app_state.pilot_moment.unwrap(),
        app_state.passenger_moment,
        app_state.baggage_moment,
        app_state.fuel_type.unwrap(),
        app_state.fuel_unit.unwrap(),
        app_state.fuel_extra.clone(),
        app_state.fuel_max.unwrap_or_default(),
        app_state.trip_duration.unwrap(),
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

pub async fn wb_chart(query: web::Query<IndexQueryParams>) -> impl Responder {
    let (app_state, _) = ApplicationState::from_query_params(query.into_inner());

    let plane = plane::build_plane(
        app_state.callsign.unwrap(),
        app_state.pilot_moment.unwrap(),
        app_state.passenger_moment,
        app_state.baggage_moment,
        app_state.fuel_type.unwrap(),
        app_state.fuel_unit.unwrap(),
        app_state.fuel_extra.clone(),
        app_state.fuel_max.unwrap_or_default(),
        app_state.trip_duration.unwrap(),
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
