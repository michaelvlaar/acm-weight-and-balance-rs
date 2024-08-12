use std::{num::ParseIntError, time::Duration};

use airplane::{
    types::{FuelType, VolumeType},
    weight_and_balance::{LeverArm, Mass, Moment, Volume},
};

use super::query_params::IndexQueryParams;

#[derive(Clone)]
pub struct ApplicationState {
    pub callsign: Option<String>,
    pub pilot_moment: Option<Moment>,
    pub passenger_moment: Option<Moment>,
    pub baggage_moment: Option<Moment>,
    pub oat: Option<f64>,
    pub pressure_altitude: Option<f64>,
    pub wind: Option<f64>,
    pub fuel_type: Option<airplane::types::FuelType>,
    pub fuel_unit: Option<airplane::types::VolumeType>,
    pub fuel_extra: Option<Mass>,
    pub fuel_max: Option<bool>,
    pub trip_duration: Option<Duration>,
    pub alternate_duration: Option<Duration>,
}

impl ApplicationState {
    pub fn apply(&self, step: &str, ctx: &mut tera::Context) {
        ctx.insert("step", step);

        if self.callsign.is_some() {
            ctx.insert("callsign", &self.callsign.clone().unwrap());
        }

        if let Some(pm) = &self.pilot_moment {
            let LeverArm::Meter(arm) = pm.lever_arm();
            let arm_str = if *arm == 5.0 / 11.0 {
                "f"
            } else if *arm == 13.0 / 22.0 {
                "b"
            } else {
                "m"
            };

            ctx.insert("pilot", &pm.mass().kilo());
            ctx.insert("pilot_seat", &arm_str);
        }

        if let Some(pm) = &self.passenger_moment {
            let LeverArm::Meter(arm) = pm.lever_arm();
            let arm_str = if *arm == 5.0 / 11.0 {
                "f"
            } else if *arm == 13.0 / 22.0 {
                "b"
            } else {
                "m"
            };

            ctx.insert("passenger", &pm.mass().kilo());
            ctx.insert("passenger_seat", &arm_str);
        }

        if let Some(bm) = &self.baggage_moment {
            ctx.insert("baggage", &bm.mass().kilo());
        }

        if self.oat.is_some() {
            ctx.insert("oat", &self.oat);
        }

        if self.pressure_altitude.is_some() {
            ctx.insert("pressure_altitude", &self.pressure_altitude);
        }

        if self.wind.is_some() {
            ctx.insert("wind", &self.wind.unwrap().abs());

            if self.wind.unwrap() >= 0.0 {
                ctx.insert("wind_direction", "headwind");
            } else {
                ctx.insert("wind_direction", "tailwind");
            }
        }

        if let Some(ft) = &self.fuel_type {
            match ft {
                FuelType::Avgas => ctx.insert("fuel_type", "avgas"),
                FuelType::Mogas => ctx.insert("fuel_type", "mogas"),
            }
        }

        if let Some(fu) = &self.fuel_unit {
            match fu {
                VolumeType::Liter => ctx.insert("fuel_unit", "liter"),
                VolumeType::Gallon => ctx.insert("fuel_unit", "gallon"),
            }
        }

        if let Some(fm) = &self.fuel_max {
            if *fm {
                ctx.insert("fuel_max", "max");
            }
        }

        if let Some(fe) = &self.fuel_extra {
            match fe {
                Mass::Mogas(v) | Mass::Avgas(v) => {
                    match v {
                        Volume::Liter(q) | Volume::Gallon(q) => ctx.insert("fuel_extra", &q),
                    }
                },
                _ => ()
            }
        }

        if let Some(d) = &self.trip_duration {
            ctx.insert("trip_duration", &duration_to_hh_mm(d));
        }

        if let Some(d) = &self.alternate_duration {
            ctx.insert("alternate_duration", &duration_to_hh_mm(d));
        }
    }

    pub fn from_query_params(params: IndexQueryParams) -> (ApplicationState, IndexQueryParams) {
        let mut state = ApplicationState {
            callsign: params.callsign.clone(),
            pilot_moment: match params.pilot {
                Some(w) => match &params.pilot_seat {
                    Some(pos) => {
                        let name = "Pilot".to_string();
                        match pos.as_str() {
                            "f" => Some(Moment::new(
                                name,
                                LeverArm::Meter(5.0 / 11.0),
                                Mass::Kilo(w),
                            )),
                            "b" => Some(Moment::new(
                                name,
                                LeverArm::Meter(13.0 / 22.0),
                                Mass::Kilo(w),
                            )),
                            _ => Some(Moment::new(
                                name,
                                LeverArm::Meter(23.0 / 44.0),
                                Mass::Kilo(w),
                            )),
                        }
                    }
                    None => None,
                },
                None => None,
            },
            passenger_moment: match &params.passenger {
                Some(ws) => {
                    let w: f64 = if ws.is_empty() {
                        0.0
                    } else {
                        ws.parse().expect("passenger weigth must be a number")
                    };

                    let name = "Passenger".to_string();

                    match &params.passenger_seat {
                        Some(pos) => match pos.as_str() {
                            "f" => Some(Moment::new(
                                name,
                                LeverArm::Meter(5.0 / 11.0),
                                Mass::Kilo(w),
                            )),
                            "b" => Some(Moment::new(
                                name,
                                LeverArm::Meter(13.0 / 22.0),
                                Mass::Kilo(w),
                            )),
                            _ => Some(Moment::new(
                                name,
                                LeverArm::Meter(23.0 / 44.0),
                                Mass::Kilo(w),
                            )),
                        },
                        None => None,
                    }
                }
                None => None,
            },
            baggage_moment: match &params.baggage {
                Some(ws) => {
                    let w: f64 = if ws.is_empty() {
                        0.0
                    } else {
                        ws.parse().expect("baggage weigth must be a number")
                    };
                    Some(Moment::new(
                        "Bagage".to_string(),
                        LeverArm::Meter(1.3),
                        Mass::Kilo(w),
                    ))
                }
                None => None,
            },
            oat: params.oat,
            pressure_altitude: params.pressure_altitude,
            wind: match params.wind_direction.clone().unwrap_or_default().as_str() {
                "headwind" => params.wind,
                "tailwind" => params.wind.map(|w| -w),
                _ => None,
            },
            fuel_type: match &params.fuel_type {
                Some(t) => match t.as_str() {
                    "mogas" => Some(airplane::types::FuelType::Mogas),
                    "avgas" => Some(airplane::types::FuelType::Avgas),
                    _ => None,
                },
                None => None,
            },
            fuel_unit: match &params.fuel_unit {
                Some(u) => match u.as_str() {
                    "liter" => Some(airplane::types::VolumeType::Liter),
                    "gallon" => Some(airplane::types::VolumeType::Gallon),
                    _ => None,
                },
                None => None,
            },
            fuel_max: params.fuel_max.as_ref().map(|m| m == "max"),
            fuel_extra: None,
            trip_duration: None,
            alternate_duration: None,
        };

        match &params.fuel_extra {
            Some(fe) => {
                let extra: f64 = fe.parse().unwrap_or_default();
                match &state.fuel_unit {
                    Some(fu) => match &state.fuel_type {
                        Some(ft) => {
                            let volume = match fu {
                                VolumeType::Liter => Volume::Liter(extra),
                                VolumeType::Gallon => Volume::Gallon(extra),
                            };

                            state.fuel_extra = Some(match ft {
                                FuelType::Mogas => Mass::Mogas(volume),
                                FuelType::Avgas => Mass::Avgas(volume),
                            })
                        }
                        None => (),
                    },
                    None => (),
                }
            }
            None => (),
        }

        match &params.trip_duration {
            Some(d) => {
                if let Ok(d) = parse_time_to_duration(d.as_str()) {
                    state.trip_duration = Some(d);
                }
            }
            None => (),
        }

        match &params.alternate_duration {
            Some(d) => {
                if let Ok(d) = parse_time_to_duration(d.as_str()) {
                    state.alternate_duration = Some(d);
                }
            }
            None => (),
        }

        (state, params)
    }
}

fn parse_time_to_duration(time_str: &str) -> Result<Duration, ParseIntError> {
    let parts: Vec<&str> = time_str.split(':').collect();

    if parts.len() != 2 {
        panic!("Verkeerd formaat, verwacht HH:mm");
    }

    let hours: u64 = parts[0].parse()?;
    let minutes: u64 = parts[1].parse()?;

    let total_seconds = (hours * 3600) + (minutes * 60);

    Ok(Duration::from_secs(total_seconds))
}

pub fn duration_to_hh_mm(duration: &Duration) -> String {
    let total_seconds = duration.as_secs();

    // Bereken uren en minuten
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;

    // Formatteer als HH:mm
    format!("{:02}:{:02}", hours, minutes)
}
