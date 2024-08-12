use crate::models::query_params::IndexQueryParams;
use actix_web::web;

pub fn parse_query(
    query: web::Query<IndexQueryParams>,
) -> (
    String,
    f64,
    String,
    f64,
    String,
    f64,
    f64,
    String,
    String,
    String,
    f64,
    f64,
    f64,
    String,
    String,
) {
    let query_params = query.into_inner();
    let callsign = query_params.callsign.expect("callsign must be present.");
    let pilot = query_params.pilot.expect("pilot should be present.");
    let passenger: f64 = query_params
        .passenger
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();
    let baggage: f64 = query_params
        .baggage
        .unwrap_or_default()
        .parse()
        .unwrap_or_default();
    let oat = query_params
        .oat
        .expect("outside air temperature should be present.");
    let pressure_altitude = query_params
        .pressure_altitude
        .expect("pressure altitude should be present.");
    let wind = query_params.wind.expect("wind should be present.");
    let wind_direction = query_params
        .wind_direction
        .unwrap_or_else(|| "headwind".to_string());
    let submit = query_params.submit.unwrap_or_default();
    let pilot_seat = query_params.pilot_seat.unwrap_or_else(|| "m".to_string());
    let passenger_seat = query_params
        .passenger_seat
        .unwrap_or_else(|| "m".to_string());
    (
        callsign,
        pilot,
        pilot_seat,
        passenger,
        passenger_seat,
        baggage,
        0.0,
        "mogas".to_string(),
        "liter".to_string(),
        "auto".to_string(),
        oat,
        pressure_altitude,
        wind,
        wind_direction,
        submit,
    )
}

