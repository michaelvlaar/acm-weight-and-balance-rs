use serde::Deserialize;

#[derive(Deserialize)]
pub struct IndexQueryParams {
    pub callsign: Option<String>,
    pub pilot: Option<f64>,
    pub pilot_seat: Option<String>,
    pub passenger: Option<String>,
    pub passenger_seat: Option<String>,
    pub baggage: Option<String>,
    pub oat: Option<f64>,
    pub pressure_altitude: Option<f64>,
    pub wind: Option<f64>,
    pub wind_direction: Option<String>,
    pub fuel_type: Option<String>,
    pub fuel_unit: Option<String>,
    pub fuel_extra: Option<String>,
    pub fuel_max: Option<String>,
    pub trip_duration: Option<String>,
    pub alternate_duration: Option<String>,
    pub reference: Option<String>,
    pub submit: Option<String>,
}

#[derive(Deserialize)]
pub struct PerfQueryParams {
    pub oat: f64,
    pub pressure_altitude: f64,
    pub mtow: f64,
    pub wind: f64,
    pub wind_direction: String,
}

#[derive(Deserialize)]
pub struct FuelOptionQueryParams {
    pub fuel_max: Option<String>,
    pub fuel_type: Option<String>,
}

#[derive(Deserialize)]
pub struct WindOptionQueryParams {
    pub wind: Option<f64>,
    pub wind_direction: Option<String>,
}
