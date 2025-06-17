use bevy::prelude::*;
use crate::objects::{ships, bodies};
enum OrbitalObjID {
    Body(bodies::BodyID),
    Ship(ships::ShipID),
}

#[derive(Component)]
pub(crate) struct OrbitingObjects(Vec<OrbitalObjID>);

impl OrbitingObjects {
    
    pub(crate) fn to_orbiting_objects(orbiting_bodies: &Vec<bodies::BodyID>) -> Self {
        let orbiting_obj = orbiting_bodies
        .iter()
        .map(|body_id| {OrbitalObjID::Body(body_id.clone())})
        .collect::<Vec<_>>();
    Self(orbiting_obj)
    }
}