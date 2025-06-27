//! A "Ship" is an object whose movement is governed by the gravitationnal
//! attraction of the celestial bodies, along with custom trajectories

use arrayvec::ArrayString;
use bevy::{math::DVec3, prelude::*, utils::HashMap};
use std::f64::consts::PI;

use crate::game::{ClearOnUnload, Loaded};
use crate::physics::influence::{HillRadius};
use crate::physics::{leapfrog::get_acceleration, G, time::TickEvent};
use crate::physics::prelude::*;
use crate::objects::{
    orbiting_obj::{OrbitingObjects, OrbitalObjID},
    bodies::BodyID,
};
use crate::ui::gui::debug_to_file;

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
            .add_systems(Update, handle_ship_events.in_set(ObjectsUpdate))
            .add_systems(OnEnter(Loaded), create_ships.in_set(ObjectsUpdate))
            .add_systems(Update, check_ship_orbits.run_if(on_event::<TickEvent>()));
    }
}
#[derive(Component)]
pub(crate) struct HostBody(pub BodyID);

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
    SwitchToOrbital{ship_id: ShipID, r_vec: Position, v_vec: Velocity, mass: Mass}
}

fn create_ships(mut commands: Commands) {
    commands.insert_resource(ShipsMapping::default());
}
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn handle_ship_events(
    mut commands: Commands,
    mut reader: EventReader<ShipEvent>,
    mut ships_mapping: ResMut<ShipsMapping>,
    mut param: ParamSet<(
        Query<(&Position, &HillRadius, &OrbitingObjects)>, 
        Query<&mut OrbitingObjects>,                       
        Query<(&Position, &HillRadius, &OrbitingObjects, &Mass)>, 
    )>,
    bodies: Query<&BodyInfo>,
    query_influenced: Query<&Influenced>,
    mapping: Res<BodiesMapping>,
    main_body: Query<&BodyInfo, With<PrimaryBody>>,
) {
    for event in reader.read() {
        match event {
            ShipEvent::Create(info) => {
                let pos = Position(info.spawn_pos);

                ships_mapping.0.entry(info.id).or_insert({
                    let influence = Influenced::new(
                        &pos,
                        &param.p0(),
                        mapping.as_ref(),
                        main_body.single().0.id,
                    );

                    commands
                        .spawn((
                            info.clone(),
                            Acceleration::new(get_acceleration(
                                info.spawn_pos,
                                param.p2() 
                                    .iter_many(&influence.influencers)
                                    .map(|(p, _, _, m)| (p.0, m.0)),
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
                if let Some(e) = ships_mapping.0.remove(id) {
                    commands.entity(e).despawn()
                }
            }
            ShipEvent::SwitchToOrbital {
                ship_id,
                r_vec,
                v_vec,
                mass,
            } => {
                if let Some(ship) = ships_mapping.0.get(ship_id) {
                    let orbit = calc_elliptical_orbit(*r_vec, *v_vec, *mass);
                    let orbiting_obj = OrbitingObjects(Vec::new());
                    let host_body_id = get_host_body(ship, &query_influenced, &bodies);
                    let host_entity = mapping.0.get(&host_body_id).unwrap();
                    let mut host_bodies = param.p1();
                    let mut host_orbiting_obj = host_bodies.get_mut(*host_entity).unwrap();                    
                    host_orbiting_obj.0.push(OrbitalObjID::Ship(*ship_id));
                    commands
                        .entity(*ship)
                        .insert((orbit, orbiting_obj.clone(), HostBody(host_body_id)));
                    commands.entity(*ship).remove::<(Acceleration, Influenced)>();
                };
            }
        }
    }
}

fn get_host_body(ship: &Entity, query_influenced: &Query<&Influenced>, bodies: &Query<&BodyInfo>) -> BodyID {
    let influenced = query_influenced.get(*ship).unwrap();
    let host_body = influenced.main_influencer.unwrap();
    bodies.get(host_body).unwrap().0.id
}

#[allow(non_snake_case)]
fn calc_elliptical_orbit(
    r_vec: Position, 
    v_vec: Velocity, 
    mass: Mass,
    ) -> EllipticalOrbit {
    let r_vec = r_vec.0;
    let v_vec = v_vec.0;
    let mu = G*mass.0;
    let v = v_vec.length();
    let r = r_vec.length();
    let h = r_vec.cross(v_vec);
    let e_vec = v_vec.cross(h)/mu - r_vec/r;
    let e = e_vec.length();
    let epsilon = v.powf(2.)/2. - mu/r;
    let semimajor_axis = -mu / (2. * epsilon);
    let inclination = (h.z/h.length()).acos();
    let n_vec = DVec3::new(-h.y, h.x, 0.);
    let mut long_asc_node;
    let mut arg_periapsis;
    if n_vec == DVec3::ZERO {
        long_asc_node = 0.;
        arg_periapsis = 0.;
    } else {
        long_asc_node = (n_vec.x/n_vec.length()).acos();
        if n_vec.y < 0. {
            long_asc_node = 2.*PI - long_asc_node;
        }
        arg_periapsis = (n_vec.dot(e_vec) / (n_vec.length() * e)).acos();
        if e_vec.z < 0. {
            arg_periapsis = 2.*PI - arg_periapsis;
        }
    }
    
    let mut true_anomaly = (e_vec.dot(r_vec) / (e * r)).acos();
    if r_vec.dot(v_vec) < 0.0 {
        true_anomaly = 2.0 * PI - true_anomaly;
    }
    let tan_half_E = ((1.0 - e) / (1.0 + e)).sqrt() * (true_anomaly / 2.0).tan();
    let E = 2.0 * tan_half_E.atan();

    let initial_mean_anomaly = E - e * E.sin();

    let revolution_period = 2.*PI*(semimajor_axis.powf(3.)/mu).powf(0.5);
    EllipticalOrbit {
        eccentricity: e,
        semimajor_axis, 
        inclination,
        long_asc_node,
        arg_periapsis,
        initial_mean_anomaly,
        revolution_period,
        mean_anomaly: initial_mean_anomaly,
        ..Default::default()
    }
} 


fn check_ship_orbits(
    ships: Query<(&ShipInfo, &Position, &Velocity, &Influenced)>,
    influencers: Query<(&Position, &Velocity, &Mass), With<HillRadius>>,
    mut writer: EventWriter<ShipEvent>,
) {
    for (info, pos, vel, influenced) in ships.iter() {
        if let Some(main_influencer) = influenced.main_influencer {
            if let Ok((inf_pos, inf_vel, inf_mass)) = influencers.get(main_influencer) {
                let r = pos.0 - inf_pos.0;
                let v = vel.0 - inf_vel.0;
                let h = r.cross(v);
                let e_vec = (v.cross(h)) / (G * inf_mass.0) - r / r.length();                
                let e = e_vec.length();
                debug_to_file("eccentricity", e);
                if e < 1.0 {
                    debug_to_file("in orbit", "!");
                    writer.send(ShipEvent::SwitchToOrbital{ship_id: info.id, r_vec: Position(r), v_vec: Velocity(v), mass: *inf_mass});
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::{
        bodies::body_data::{BodyData, BodyType},
        orbiting_obj::{OrbitingObjects, OrbitalObjID},
        id::id_from,
    };
    use bevy::ecs::system::SystemState;
    use crate::physics::SECONDS_PER_DAY;
    const TWO_PI: f64 = std::f64::consts::TAU; 

    
    fn setup(app: &mut App, info: &ShipInfo) -> Entity {

        app.add_event::<ShipEvent>();

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

        let mut state_mapping: SystemState<Res<BodiesMapping>> = SystemState::new(app.world_mut());
        let mut state_query: SystemState<Query<(&Position, &HillRadius, &OrbitingObjects)>> = SystemState::new(app.world_mut());
        let (bodies_mapping, bodies) = {
            let world = app.world();
            (
                state_mapping.get(world),
                state_query.get(world)
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

        ship_entity
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
        app.add_systems(Update, check_ship_orbits);
        app.add_event::<ShipEvent>();

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
        
        // Vaisseau dans le rayon de Hill mais pas en orbite 
        let info = ShipInfo {
            id: ShipID::from("s2").unwrap(),
            spawn_pos: DVec3 { x: -2522401.726568888, y: 142515717.88224745, z: 0.},
            spawn_speed: DVec3 { x: -25224010.7265688, y: -544246.5886227646, z: 0. } 
        };
        let ship_entity = setup(&mut app, &info);

        app.add_systems(Update, handle_ship_events);
        app.add_systems(Update, check_ship_orbits);

        app.update();

        let orbit = app.world().get::<EllipticalOrbit>(ship_entity);
        assert!(orbit.is_none(), "Le vaisseau ne devrait pas avoir de composant EllipticalOrbit");

    }

    #[test]
    fn test_calc_elliptical_orbit() {
    // r and v found at https://ssd.jpl.nasa.gov/api/horizons.api?format=json&COMMAND=399&CENTER=%27@sun%27&EPHEM_TYPE=VECTORS&START_TIME=%272000-01-01%27&STOP_TIME=%272000-01-02%27&STEP_SIZE=%271%20d%27&OUT_UNITS=KM-S
    let r_vec = DVec3::new(
    -2.521092855899356e7,
    1.449279195838006e8,
    -6.164165719002485e2,
    );
    let v_vec = DVec3::new(
        -29.83983333677879,
        -5.207633902410673,
        6.16844118423998e-05,
    )* SECONDS_PER_DAY;

    let earth_mass = 1.9885e30;

    let orbit = calc_elliptical_orbit(
        Position(r_vec),
        Velocity(v_vec),
        Mass(earth_mass),
    );
    let semimajor =  149598023.;
    let eccentricity = 0.01670;
    let inclination = 0.;
    let long_asc_node = 18.272;
    let arg_periapsis = 85.901;
    // let initial_mean_anomaly = 358.617;
    let revolution_period = 365.256;

    let tolerance_prct = 0.2;
    let epsilon = 1e-5;

    fn normalize_angle(angle: f64) -> f64 {
        let mut a = angle % TWO_PI;
        if a < 0.0 {
            a += TWO_PI;
        }
        a
    }

    assert!((orbit.semimajor_axis - semimajor).abs() < tolerance_prct * semimajor);
    assert!((orbit.eccentricity - eccentricity).abs() < eccentricity * tolerance_prct);
    assert!((orbit.inclination - inclination).abs() < epsilon);
    assert!((normalize_angle(orbit.long_asc_node) - normalize_angle(long_asc_node)).abs() < long_asc_node * tolerance_prct);
    assert!((normalize_angle(orbit.arg_periapsis) - normalize_angle(arg_periapsis)).abs() < arg_periapsis * tolerance_prct);
    assert!((orbit.revolution_period - revolution_period).abs() < revolution_period * tolerance_prct);

    }
}