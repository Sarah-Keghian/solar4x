//! A "Ship" is an object whose movement is governed by the gravitationnal
//! attraction of the celestial bodies, along with custom trajectories

use arrayvec::ArrayString;
use bevy::{math::DVec3, prelude::*, utils::HashMap};

use crate::game::{ClearOnUnload, Loaded};
use crate::physics::influence::HillRadius;
use crate::physics::leapfrog::get_acceleration;
use crate::physics::prelude::*;

use super::id::MAX_ID_LENGTH;
use super::prelude::{BodiesMapping, BodyInfo, PrimaryBody};
use super::ObjectsUpdate;
use scheduler::{ShipSchedule};

pub mod trajectory;
pub mod scheduler;

// pub(crate) struct ShipID(u64);

// #[derive(Resource, Default)]
// struct ShipIDBuilder(NumberIncrementer);

// impl IDBuilder for ShipIDBuilder {
//     type ID = ShipID;

//     fn incrementer(&mut self) -> &mut NumberIncrementer {
//         &mut self.0
//     }

//     fn id_from_u64(u: u64) -> Self::ID {
//         ShipID(u)
//     }
// }
pub struct ShipsPlugin;

impl Plugin for ShipsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(trajectory::plugin)
            .add_event::<ShipEvent>()
            .add_systems(Update, handle_ship_events.in_set(ObjectsUpdate))
            .add_systems(OnEnter(Loaded), create_ships.in_set(ObjectsUpdate));
    }
}

pub type ShipID = ArrayString<MAX_ID_LENGTH>;

#[derive(Component, Clone, Default, PartialEq)]
pub struct ShipInfo {
    pub id: ShipID,
    pub spawn_pos: DVec3,
    pub spawn_speed: DVec3,
}

#[derive(Resource, Default)]
pub struct ShipsMapping(pub HashMap<ShipID, Entity>);

#[derive(Event)]
pub enum ShipEvent {
    Create(ShipInfo),
    Remove(ShipID),
}

fn create_ships(mut commands: Commands) {
    commands.insert_resource(ShipsMapping::default());
}

fn handle_ship_events(
    mut commands: Commands,
    mut reader: EventReader<ShipEvent>,
    mut ships: ResMut<ShipsMapping>,
    bodies: Query<(&Position, &HillRadius, &BodyInfo)>,
    mapping: Res<BodiesMapping>,
    main_body: Query<&BodyInfo, With<PrimaryBody>>,
) {
    for event in reader.read() {
        match event {
            ShipEvent::Create(info) => {
                let pos = Position(info.spawn_pos);
                ships.0.entry(info.id).or_insert({
                    let influence =
                        Influenced::new(&pos, &bodies, mapping.as_ref(), main_body.single().0.id);
                    commands
                        .spawn((
                            info.clone(),
                            Acceleration::new(get_acceleration(
                                info.spawn_pos,
                                bodies
                                    .iter_many(&influence.influencers)
                                    .map(|(p, _, i)| (p.0, i.0.mass)),
                            )),
                            influence,
                            pos,
                            Velocity(info.spawn_speed),
                            ShipSchedule(Vec::new()),
                            TransformBundle::from_transform(Transform::from_xyz(0., 0., 1.)),
                            ClearOnUnload,
                        ))
                        .id()
                });
            }
            ShipEvent::Remove(id) => {
                if let Some(e) = ships.0.remove(id) {
                    commands.entity(e).despawn()
                }
            }
        }
    }
}
