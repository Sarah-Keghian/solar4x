use std::collections::BTreeMap;

use bevy::{ecs::system::QueryLens, math::DVec3, prelude::*, utils::HashMap};

use crate::{
    objects::{prelude::*, ships::trajectory::ManeuverNode},
    physics::prelude::*,
    utils::algebra::orbital_to_global_matrix,
};

use super::{
    influence::HillRadius,
    leapfrog::{get_acceleration, get_dv, get_dx},
    time::{GAMETIME_PER_SIMTICK, SIMTICKS_PER_TICK},
};
use crate::objects::orbiting_obj::{OrbitalObjID, OrbitingObjects};

/// Number of client updates between two predictions
pub const PREDICTIONS_STEP: usize = 20;

/// A component representing identifying a prediction of a ship at a selected time
#[derive(Component, Clone, Copy)]
pub struct Prediction {
    pub ship: Entity,
    pub index: usize,
    pub simtick: u64,
}

#[derive(Debug)]
pub struct PredictionStart {
    pub pos: DVec3,
    pub speed: DVec3,
    pub acc: DVec3,
    pub simtick: u64,
}

impl PredictionStart {
    /// Compute the future positions of this point with respect to a given referential and considering some influencer's gravitationnal pull on it
    pub fn compute_predictions(
        &self,
        number: usize,
        influence: &Influenced,
        reference: Option<Entity>,
        bodies: &mut QueryLens<(&EllipticalOrbit, &BodyInfo, &HillRadius)>,
        mapping: &HashMap<BodyID, Entity>,
        nodes: &BTreeMap<u64, ManeuverNode>,
    ) -> Vec<(DVec3, DVec3)> {
        let dt = GAMETIME_PER_SIMTICK;
        let mut bodies = bodies.query();
        let mut pos = self.pos;
        let mut speed = self.speed;
        let mut predictions = Vec::new();
        let simulated = simulated_from_influence(
            influence,
            &mut bodies.transmute_lens::<&OrbitingObjects>(),
            mapping,
        );
        let mut map = simulated
            .iter()
            .map(|e| (*e, (DVec3::ZERO, DVec3::ZERO, bodies.get(*e).unwrap().2 .0)))
            .collect::<HashMap<_, _>>();
        let mut influencers = influence
            .influencers
            .iter()
            .map(|e| {
                let comp = bodies.get(*e).unwrap();
                (*e, comp.1 .0.mass)
            })
            .collect::<HashMap<_, _>>();
        let mut main = influence.main_influencer;
        let initial_bodies_coords = get_bodies_coordinates(
            map.keys().cloned(),
            &mut bodies.transmute_lens::<(&EllipticalOrbit, &BodyInfo)>(),
            mapping,
            self.simtick,
        );
        map.values_mut()
            .enumerate()
            .for_each(|(i, v)| (v.0, v.1) = initial_bodies_coords[i]);
        let initial_ref_coords = reference.and_then(|r| map.get(&r).cloned()).unwrap_or((
            DVec3::ZERO,
            DVec3::ZERO,
            f64::INFINITY,
        ));
        let mut acc = self.acc;
        let mut previous_acc;
        for i in 1..number + 1 {
            let simtick = self.simtick + i as u64;
            let bodies_coords = get_bodies_coordinates(
                map.keys().cloned(),
                &mut bodies.transmute_lens::<(&EllipticalOrbit, &BodyInfo)>(),
                mapping,
                simtick,
            );
            map.values_mut()
                .enumerate()
                .for_each(|(i, v)| (v.0, v.1) = bodies_coords[i]);

            if simtick % SIMTICKS_PER_TICK == 0 {
                if let Some(node) = nodes.get(&(simtick / SIMTICKS_PER_TICK)) {
                    // For now, the origin body must be simulated
                    if let Some(node_origin) = mapping.get(&node.origin) {
                        if let Some(&(origin_pos, origin_speed, _)) = map.get(node_origin) {
                            speed += orbital_to_global_matrix(origin_pos, origin_speed, pos, speed)
                                * node.thrust;
                        }
                    }
                }

                let (new_main, new_radius) = map
                    .iter()
                    .filter_map(|(e, (body_pos, _, r))| {
                        let dist = (*body_pos - pos).length();
                        if dist > *r {
                            None
                        } else {
                            Some((*e, *r))
                        }
                    })
                    .min_by(|(_, a), (_, b)| a.total_cmp(b))
                    .unwrap();
                if let Some(main_entity) = main {
                    if new_main != main_entity {
                        let radius = map.get(&main_entity).unwrap().2;
                        let new_children = children_entities(
                            new_main,
                            &mut bodies.transmute_lens::<&OrbitingObjects>(),
                            mapping,
                        );
                        map.retain(|k, _| {
                            influencers.contains_key(k) || [new_main, main_entity].contains(k)
                        });
                        map.extend(
                            new_children.into_iter().map(|e| {
                                (e, (DVec3::ZERO, DVec3::ZERO, bodies.get(e).unwrap().2 .0))
                            }),
                        );
                        if radius > new_radius {
                            influencers.insert(new_main, bodies.get(new_main).unwrap().1 .0.mass);
                        } else {
                            influencers.remove(&main_entity);
                        }
                        main = Some(new_main);
                    }
                }
            }

            pos += get_dx(speed, acc, dt);
            let ref_coords = reference.and_then(|r| map.get(&r).cloned()).unwrap_or((
                DVec3::ZERO,
                DVec3::ZERO,
                f64::INFINITY,
            ));
            previous_acc = acc;
            acc = get_acceleration(pos, influencers.iter().map(|(e, m)| (map[e].0, *m)));
            speed += get_dv(previous_acc, acc, dt);
            predictions.push((
                pos - ref_coords.0 + initial_ref_coords.0,
                speed - ref_coords.1,
            ));
        }
        predictions
    }
}

fn simulated_from_influence(
    influence: &Influenced,
    bodies: &mut QueryLens<&OrbitingObjects>,
    bodies_mapping: &HashMap<BodyID, Entity>,
) -> Vec<Entity> {
    let mut v = influence.influencers.clone();
    if let Some(main) = influence.main_influencer {
        v.extend(children_entities(main, bodies, bodies_mapping));
    }
    v
}

fn children_entities(
    parent: Entity,
    objects: &mut QueryLens<&OrbitingObjects>,
    bodies_mapping: &HashMap<BodyID, Entity>,
) -> Vec<Entity> {
    let query = objects.query();
    query
        .get(parent)
        .unwrap()
        .0
        .iter()
        .filter_map(|orbital_obj| {
            let id = match orbital_obj {
                OrbitalObjID::Body(body_id) => body_id,
                OrbitalObjID::Ship(ship_id) => ship_id,
            };
            bodies_mapping.get(id).cloned()
        } )
        .collect()
}

pub fn get_bodies_coordinates(
    selected_bodies: impl Iterator<Item = Entity>,
    bodies: &mut QueryLens<(&EllipticalOrbit, &BodyInfo)>,
    mapping: &HashMap<BodyID, Entity>,
    tick: u64,
) -> Vec<(DVec3, DVec3)> {
    fn compute_pos_rec(
        e: Entity,
        map: &mut HashMap<Entity, (DVec3, DVec3)>,
        tick: u64,
        bodies: Query<(&EllipticalOrbit, &BodyInfo)>,
        mapping: &HashMap<BodyID, Entity>,
    ) -> (DVec3, DVec3) {
        let (orbit, BodyInfo(data)) = bodies.get(e).unwrap();
        let mut o = orbit.clone();
        o.update_pos(tick as f64 * GAMETIME_PER_SIMTICK);
        if let Some(coords) = map.get(&e) {
            *coords
        } else {
            let (pos, speed) = data.host_body.map_or((DVec3::ZERO, DVec3::ZERO), |parent| {
                let (parent_pos, parent_speed) =
                    compute_pos_rec(mapping[&parent], map, tick, bodies, mapping);
                (parent_pos + o.local_pos, parent_speed + o.local_speed)
            });
            map.insert(e, (pos, speed));
            (pos, speed)
        }
    }

    let mut map = HashMap::new();

    selected_bodies
        .map(|e| compute_pos_rec(e, &mut map, tick, bodies.query(), mapping))
        .collect()
}

#[cfg(test)]
mod tests {
    use bevy::{ecs::system::SystemState, prelude::*};

    use crate::{
        physics::leapfrog::get_acceleration, prelude::*, utils::algebra::circular_orbit_around_body,
    };

    use super::*;

    #[test]
    fn test_predictions() {
        let mut app = App::new();
        app.add_plugins(ClientPlugin::testing().in_mode(ClientMode::Singleplayer));
        app.update();
        let world = app.world_mut();
        let mapping = &world.resource::<BodiesMapping>().0;
        let earth = *mapping.get(&id_from("terre")).unwrap();
        let sun = *mapping.get(&id_from("soleil")).unwrap();
        let (&mass, &earth_pos, &earth_speed) = world
            .query::<(&Mass, &Position, &Velocity)>()
            .get(world, earth)
            .unwrap();
        let (pos, speed) = circular_orbit_around_body(1e5, mass.0, earth_pos.0, earth_speed.0);
        let influencers = vec![sun, earth];
        let influence = Influenced {
            main_influencer: Some(earth),
            influencers: influencers.clone(),
        };
        #[allow(clippy::type_complexity)]
        let mut system_state: SystemState<(
            Res<BodiesMapping>,
            Query<(&EllipticalOrbit, &BodyInfo, &HillRadius)>,
            Query<(&Position, &Mass)>,
        )> = SystemState::new(world);
        let (mapping, mut bodies, query) = system_state.get(world);
        let predictions = PredictionStart {
            pos,
            speed,
            simtick: 0,
            acc: get_acceleration(pos, query.iter_many(&influencers).map(|(p, m)| (p.0, m.0))),
        }
        .compute_predictions(
            3,
            &influence,
            Some(earth),
            &mut bodies.as_query_lens(),
            &mapping.0,
            &BTreeMap::new(),
        );
        for (i, (p, _)) in predictions.into_iter().enumerate() {
            // dbg!(p);
            // dbg!(pos + (i + 1) as f64 * (speed - earth_speed.0) * GAMETIME_PER_SIMTICK);
            assert!(
                (p - (pos + (i + 1) as f64 * (speed - earth_speed.0) * GAMETIME_PER_SIMTICK))
                    .length()
                    <= 5e4
            );
        }
    }
}
