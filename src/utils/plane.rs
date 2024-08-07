use airplane::weight_and_balance::{
    Airplane, CenterOfGravity, LeverArm, Limits, Mass, Moment, Volume,
};

fn calculate_lever_arm(seat: &str) -> LeverArm {
    match seat {
        "f" => LeverArm::Meter(5.0 / 11.0),
        "m" => LeverArm::Meter(23.0 / 44.0),
        _ => LeverArm::Meter(13.0 / 22.0),
    }
}

pub fn build_plane(
    callsign: String,
    pilot: f64,
    pilot_seat: String,
    passenger: f64,
    passenger_seat: String,
    baggage: f64,
    fuel_quantity: f64,
    fuel_type: String,
    fuel_quantity_type: String,
    fuel_option: String,
) -> Airplane {
    let empty_mass = if callsign == "PHDHA" { 517.0 } else { 529.5 };

    let mut plane = Airplane::new(
        callsign.clone(),
        vec![
            Moment::new(
                "Empty Mass".to_string(),
                LeverArm::Meter(0.4294),
                Mass::Kilo(empty_mass),
            ),
            Moment::new(
                "Pilot".to_string(),
                calculate_lever_arm(&pilot_seat),
                Mass::Kilo(pilot),
            ),
            Moment::new(
                "Passenger".to_string(),
                calculate_lever_arm(&passenger_seat),
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
