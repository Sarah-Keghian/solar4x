use bevy::prelude::*;
use crate::objects::{ships, bodies};
use crate::objects::bodies::main_bodies::MainBodyData;

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum OrbitalObjID {
    Body(bodies::BodyID),
    Ship(ships::ShipID),
}

#[derive(Component, PartialEq, Debug, Clone)]
pub(crate) struct OrbitingObjects(pub(crate) Vec<OrbitalObjID>);

impl From<&MainBodyData> for OrbitingObjects {
    fn from(value: &MainBodyData) -> Self {
        let orbiting_bodies: Vec<_> = value
            .orbiting_bodies
            .iter()
            .map(|b| Into::<bodies::BodyID>::into(b.clone()))            
            .collect();

        let orbiting_obj = orbiting_bodies
            .iter()
            .map(|body_id| {OrbitalObjID::Body(body_id.clone())})
            .collect::<Vec<_>>();

        Self(orbiting_obj)
    }
}
