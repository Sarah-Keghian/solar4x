//! A "Ship" is an object whose movement is governed by the gravitationnal
//! attraction of the celestial bodies, along with custom trajectories

use arrayvec::ArrayString;
use bevy::{math::DVec3, prelude::*, utils::HashMap};
use std::f64::consts::PI;

use crate::game::{ClearOnUnload, Loaded};
use crate::physics::influence::{HillRadius};
use crate::physics::{leapfrog::get_acceleration, G};
use crate::physics::prelude::*;
use crate::objects::orbiting_obj::OrbitingObjects;

use super::id::MAX_ID_LENGTH;
use super::prelude::{BodiesMapping, BodyInfo, PrimaryBody};
use super::ObjectsUpdate;

pub mod trajectory;

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
            .add_event::<SwitchToOrbitalError>()
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
    SwitchToOrbital{
        ship_id: ShipID,
    },
}

#[derive(Event)]
pub struct SwitchToOrbitalError {
    pub message: String,
}

fn create_ships(mut commands: Commands) {
    commands.insert_resource(ShipsMapping::default());
}

fn handle_ship_events(
    mut commands: Commands,
    mut reader: EventReader<ShipEvent>,
    mut ships: ResMut<ShipsMapping>,
    mut error_writer: EventWriter<SwitchToOrbitalError>,
    bodies: Query<(&Position, &HillRadius, &OrbitingObjects, &Mass)>,
    query_influenced: Query<(&Position, &HillRadius, &OrbitingObjects)>,
    mapping: Res<BodiesMapping>,
    main_body: Query<&BodyInfo, With<PrimaryBody>>,
    ship_query: Query<(&Position, &Velocity, &Influenced)>,
    influencer_query: Query<(&Position, &Velocity, &Mass), With<HillRadius>> 
    ) {
    for event in reader.read() {
        match event {
            ShipEvent::Create(info) => {
                let pos = Position(info.spawn_pos);
                ships.0.entry(info.id).or_insert({
                    let influence =
                        Influenced::new(&pos, &query_influenced, mapping.as_ref(), main_body.single().0.id);
                    commands
                        .spawn((
                            info.clone(),
                            Acceleration::new(get_acceleration(
                                info.spawn_pos,
                                bodies
                                    .iter_many(&influence.influencers)
                                    .map(|(p, _, _,m)| (p.0, m.0)),
                            )),
                            influence,
                            pos,
                            Velocity(info.spawn_speed),
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
            ShipEvent::SwitchToOrbital { ship_id } => {
                if let Some(ship) = ships.0.get(ship_id) {
                    if let Some(orbit) = calc_elliptical_orbit(*ship, &ship_query, &influencer_query) {
                        commands.entity(*ship).insert(orbit);
                        commands.entity(*ship).remove::<(Acceleration, Influenced)>();      
                    } else {
                        error_writer.send(SwitchToOrbitalError { message: format!("Le vaisseau {:?} n'est pas en orbite", ship_id) });
                    }
                };
            }
        }
    }
}

fn calc_elliptical_orbit(
    ship: Entity,
    ship_query: &Query<(&Position, &Velocity, &Influenced)>,
    influencer_query: &Query<(&Position, &Velocity, &Mass), With<HillRadius>>
    ) -> Option<EllipticalOrbit> {
    if let Some((r_vec, v_vec, mass)) = find_host_body(ship, &ship_query, &influencer_query) {
        let mu = G*mass.0;
        let v = v_vec.length();
        let r = r_vec.length();
        let h = r_vec.cross(v_vec);
        let e_vec = v_vec.cross(h)/mu - r_vec/r;
        let e = e_vec.length();
        let epsilon = v.powf(2.)/2. - mu/r;
        let semimajor_axis = -mu/2.*epsilon;
        let inclination = (h.z/h.length()).acos();
        let n_vec = DVec3::new(-h.y, h.x, 0.);
        let mut long_asc_node = (n_vec.x/n_vec.length()).acos();
        if n_vec.y < 0. {
            long_asc_node = 2.*PI - long_asc_node;
        }
        let mut arg_periapsis = (n_vec.dot(r_vec)/e*r).acos();
        if e_vec.z < 0. {
            arg_periapsis = 2.*PI - arg_periapsis;
        }
        let mut initial_mean_anomaly = (e_vec.dot(r_vec)/e*r).acos();
        if r_vec.dot(v_vec) < 0. {
            initial_mean_anomaly = 2.*PI - initial_mean_anomaly;
        }
        let revolution_period = 2.*PI*(semimajor_axis.powf(3.)/mu).powf(0.5);

        Some(EllipticalOrbit {
            eccentricity: e,
            semimajor_axis, 
            inclination,
            long_asc_node,
            arg_periapsis,
            initial_mean_anomaly,
            revolution_period,
            mean_anomaly: initial_mean_anomaly,
            ..Default::default()
        })
    } 
    else {
        return None; 
    }
}

fn find_host_body(
    ship: Entity, 
    ship_query: &Query<(&Position, &Velocity, &Influenced)>, 
    influencer_query: &Query<(&Position, &Velocity, &Mass), With<HillRadius>>) -> Option<(DVec3, DVec3, Mass)>
    {
    if let Ok((ship_pos, ship_vel, influenced)) = ship_query.get(ship) {
        for influencer in &influenced.influencers {
            if let Ok((body_pos, body_vel, body_mass)) = influencer_query.get(*influencer) {
                let r = ship_pos.0 - body_pos.0;
                let v = ship_vel.0 - body_vel.0;
                let h = r.cross(v);
                let e_vec = v.cross(h)/G*body_mass.0 - r/r.length();
                let e = e_vec.length();
                if e >= 1.0 {
                    return Some((r, v, *body_mass));
                } else {
                    continue;
                }
            }; 
        }
    };
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::{
        bodies::body_data::{BodyData, BodyType},
        orbiting_obj::{OrbitingObjects, OrbitalObjID},
    };
    use bevy::ecs::system::SystemState;
    use crate::objects::id::id_from;
    
    
    fn setup(app: &mut App, info: &ShipInfo) -> Entity {

        app.add_event::<ShipEvent>();
        app.add_event::<SwitchToOrbitalError>();

        let sun_data = BodyData {
            id: id_from("soleil"),
            name: "Sun".into(),
            body_type: BodyType::Star,
            host_body: None,
            semimajor_axis: 0.,
            eccentricity: 0.,
            inclination: 0.,
            long_asc_node: 0.,
            arg_periapsis: 0.,
            initial_mean_anomaly: 0.,
            periapsis: 0.,
            apoapsis: 0.,
            revolution_period: 0.,
            rotation_period: 0.,
            radius: 695508.,
            mass: 1.989e30
        };
        let earth_data = BodyData {
            id: id_from("terre"),
            name: "Earth".into(),
            body_type: BodyType::Planet,
            host_body: Some(id_from("terre")),
            semimajor_axis: 149598023.,
            eccentricity: 0.01670,
            inclination: 0.,
            long_asc_node: 18.272,
            arg_periapsis: 85.901,
            initial_mean_anomaly: 358.617,
            periapsis: 147095000.,
            apoapsis: 152100000.,
            revolution_period: 365.256,
            rotation_period: 23.9345,
            radius: 6371.00840,
            mass: 5.97237e24
        };
        let primary_body = app.world_mut().spawn( (
            Position::default(),
            EllipticalOrbit::from(&sun_data),
            Mass(sun_data.mass),
            OrbitingObjects(vec![OrbitalObjID::Body(id_from("terre"))]),
            BodyInfo(sun_data.clone()),
            Velocity::default(),
            ClearOnUnload,
            PrimaryBody,
            HillRadius(f64::INFINITY)
        )).id();

        let hill_radius_earth = (earth_data.semimajor_axis
            * (1. - earth_data.eccentricity)
            * (earth_data.mass / (3. * (sun_data.mass + earth_data.mass))).powf(1. / 3.))
            .max(earth_data.radius);    
        
        let body = app.world_mut().spawn( (
            Position::default(),
            EllipticalOrbit::from(&earth_data),
            Mass(earth_data.mass),
            OrbitingObjects(Vec::new()),
            BodyInfo(earth_data),
            Velocity::default(),
            ClearOnUnload,
            HillRadius(hill_radius_earth)
        )).id();

        let mut mapping = HashMap::new();
        mapping.insert(id_from("soleil"), primary_body);
        mapping.insert(id_from("terre"), body);
        app.insert_resource(ShipsMapping::default());
        app.insert_resource(BodiesMapping(mapping));

        let mut state_mapping: SystemState<Res<BodiesMapping>> = SystemState::new(&mut app.world_mut());
        let mut state_query: SystemState<Query<(&Position, &HillRadius, &OrbitingObjects)>> = SystemState::new(&mut app.world_mut());
        let (bodies_mapping, bodies) = {
            let world = app.world();
            (
                state_mapping.get(&world),
                state_query.get(&world)
            )
        };

        let pos = Position(info.spawn_pos); 
        let influence = Influenced::new(&pos, &bodies, bodies_mapping.as_ref(), id_from("soleil"));
        let acceleration = Acceleration{current: DVec3::ZERO, previous: DVec3::ZERO};

        let ship_entity = app.world_mut().spawn((
            info.clone(),
            acceleration,
            influence,
            pos,
            Velocity(info.spawn_speed),
            TransformBundle::from_transform(Transform::from_xyz(0., 0., 1.)),
            ClearOnUnload,
        )).id();

        app.world_mut().resource_mut::<ShipsMapping>().0.insert(info.id, ship_entity);

        return ship_entity;
    }

    #[test]
    fn test_switch_to_orbital() {

        let mut app = App::new();

        let info = ShipInfo {
            id: ShipID::from("s").unwrap(),
            spawn_pos: DVec3 { x: -32501208.838173263, y: 143561259.9263618, z: 0.},
            spawn_speed: DVec3 { x: -2696715.3893552525, y: -672187.3782865074, z: 0. } 
        };

        let ship_entity = setup(&mut app, &info);

        app.add_systems(Update, handle_ship_events);

        app.world_mut().resource_mut::<Events<ShipEvent>>()
            .send(ShipEvent::SwitchToOrbital { ship_id: info.id });

        app.update();

        let orbit = app.world().get::<EllipticalOrbit>(ship_entity);
        assert!(orbit.is_some(), "Le vaisseau devrait maintenant avoir un composant EllipticalOrbit");

    }

    #[test]
    fn test_switch_to_orbital_impossible() {

        let mut app = App::new();

        // let info = ShipInfo {
        //     id: ShipID::from("s1").unwrap(),
        //     spawn_pos: DVec3 { x: -7543652.249402, y: 129905998.4027889, z: 0.},
        //     spawn_speed: DVec3 { x: -2304513.078405577, y: -1267321.762409748, z: 0. } 
        // };
        
        //Vaisseau dans le rayon de Hill mais pas en orbite 
        let info = ShipInfo {
            id: ShipID::from("s2").unwrap(),
            spawn_pos: DVec3 { x: -2522401.726568888, y: 142515717.88224745, z: 0.},
            spawn_speed: DVec3 { x: -2522401.726568888, y: -544246.5886227646, z: 0. } 
        };

        let ship_entity = setup(&mut app, &info);

        app.add_systems(Update, handle_ship_events);

        let orbit = app.world().get::<EllipticalOrbit>(ship_entity);
        assert!(orbit.is_none(), "Le vaisseau ne devrait pas avoir de composant EllipticalOrbit");

    }
}