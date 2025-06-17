use bevy::prelude::SystemSet;

pub mod bodies;
pub mod id;
pub mod ships;
pub mod orbital_obj;

pub mod prelude {

    pub use super::bodies::{
        bodies_config::BodiesConfig,
        body_data::{BodyData, BodyType},
        BodiesMapping, BodyID, BodyInfo, PrimaryBody,
    };
    pub use super::id::id_from;
    pub use super::ships::{ShipEvent, ShipID, ShipInfo, ShipsMapping};
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ObjectsUpdate;
