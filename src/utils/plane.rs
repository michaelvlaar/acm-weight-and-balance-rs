use std::time::Duration;

use airplane::{
    types::{FuelType, VolumeType},
    weight_and_balance::{Airplane, CenterOfGravity, LeverArm, Limits, Mass, Moment, Volume},
};

pub fn build_plane(
    callsign: String,
    pilot_moment: Moment,
    passenger_moment: Option<Moment>,
    baggage_moment: Option<Moment>,
    fuel_type: FuelType,
    fuel_unit: VolumeType,
    fuel_extra: Option<Mass>,
    fuel_max: bool,
    trip_duration: Duration,
) -> Airplane {
    let empty_mass = if callsign == "PHDHA" { 517.0 } else { 529.5 };

    let mut moments = vec![
        Moment::new(
            "Empty Mass".to_string(),
            LeverArm::Meter(0.4294),
            Mass::Kilo(empty_mass),
        ),
        pilot_moment,
    ];

    if let Some(m) = passenger_moment {
        moments.push(m);
    }

    if let Some(m) = baggage_moment {
        moments.push(m);
    }

    let mut plane = Airplane::new(
        callsign,
        moments,
        Limits::new(
            Mass::Kilo(558.0),
            Mass::Kilo(750.0),
            CenterOfGravity::Millimeter(427.0),
            CenterOfGravity::Millimeter(523.0),
        ),
        Volume::Liter(17.0 * trip_duration.as_secs_f64() / 60.0 / 60.0),
    );

    let fuel_name = "Fuel".to_string();
    let fuel_lever_arm = LeverArm::Meter(0.325);

    if fuel_max {
        plane.add_max_fuel_within_limits(
            fuel_name,
            fuel_lever_arm,
            fuel_type,
            fuel_unit,
            Some(Volume::Liter(110.0)),
        );
    } else {
        plane.add_moment(Moment::new(
            fuel_name,
            fuel_lever_arm,
            fuel_extra.expect("total fuel should be present"), 
        ));
    }

    plane
}
