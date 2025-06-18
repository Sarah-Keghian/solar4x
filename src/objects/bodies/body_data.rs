use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::BodyID;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Default, PartialOrd)]
pub enum BodyType {
    Star,
    #[default]
    Planet,
    Moon,
    #[serde(alias = "Dwarf Planet")]
    DwarfPlanet,
    Asteroid,
    Comet,
}

impl Display for BodyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Star => "Star",
            Self::Planet => "Planet",
            Self::Moon => "Moon",
            Self::Asteroid => "Asteroid",
            Self::DwarfPlanet => "Dwarf Planet",
            Self::Comet => "Comet",
        })
    }
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct BodyData {
    pub id: BodyID,
    pub name: String,
    pub body_type: BodyType,
    pub host_body: Option<BodyID>,

    // Orbital elements
    pub semimajor_axis: f64,
    pub eccentricity: f64,
    pub inclination: f64,
    pub long_asc_node: f64,
    pub arg_periapsis: f64,
    pub initial_mean_anomaly: f64,

    pub periapsis: f64,
    pub apoapsis: f64,

    // Time
    // Time required to complete a cycle around the host body (in earth days)
    pub revolution_period: f64,
    // Time required to rotate around itself (in earth hours)
    pub rotation_period: f64,

    pub radius: f64,
    pub mass: f64,
}
