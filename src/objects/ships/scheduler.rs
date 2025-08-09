use super::trajectory::{TrajectoryEvent, ManeuverNode};
use crate::objects::ships::{ShipID};
use crate::physics::time::GameTime;
use crate::prelude::{ShipInfo, ShipsMapping};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, handle_schedules);
    app.add_systems(Update, handle_add_action_to_schedule);
    app.add_event::<AddAction>();
}

#[derive(Clone)]
pub(crate) enum ShipActionKind {
    AddNode{ node: ManeuverNode },
    // OtherAction,
}

#[derive(Component)]
pub(crate) struct ShipSchedule (pub(crate) Vec<(u64, ShipActionKind)>);

#[derive(Event)]
pub(crate) struct AddAction{pub ship_id: ShipID, pub tick: u64, pub action: ShipActionKind}

fn handle_schedules (
    mut query: Query<(&mut ShipSchedule, &ShipInfo)>,
    mut traj_writer:EventWriter<TrajectoryEvent>,
    time: Res<GameTime>, 
) {
    for (mut schedule, info) in query.iter_mut() {
        let mut i: usize = 0;
        while i < schedule.0.len() {
            if schedule.0[i].0 <= time.tick() {
                let (tick, kind) = schedule.0.remove(i);
                let ship_id = info.id;
                convert_kind(tick, &kind, &ship_id, &mut traj_writer);
            }
            i += 1;
        }
    }
}

fn convert_kind(
    tick: u64,
    kind: &ShipActionKind,
    ship: &ShipID,
    traj_writer: &mut EventWriter<TrajectoryEvent>
) {
    match kind {
        ShipActionKind::AddNode{node} => {traj_writer.send(TrajectoryEvent::AddNode {
                                                            ship: *ship,
                                                            node: node.clone(),
                                                            tick,
                                                            }
                                                        );
        },
     // ShipActionKind::OtherAction => ...
    }
}

fn handle_add_action_to_schedule(
    mut reader: EventReader<AddAction>,
    mut query: Query<&mut ShipSchedule>,
    ships_map: Res<ShipsMapping>,
) {
    for event in reader.read() {
        if let Some(&entity) = ships_map.0.get(&event.ship_id) {
            if let Ok(mut schedule) = query.get_mut(entity) {
                schedule.0.push((event.tick, event.action.clone()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::{prelude::*, math::DVec3};
    use super::*;
    use arrayvec::ArrayString;
    use crate::physics::time;

    #[test]
    fn test_handle_schedules() {
        let mut app = App::new();
        
        app.add_event::<TrajectoryEvent>()
            .insert_resource(Time::<Fixed>::from_hz(64.))
            .add_systems(FixedUpdate, handle_schedules)
            .init_resource::<time::GameTime>()
            .init_resource::<time::SimStepSize>()
            .add_systems(FixedUpdate, time::update_simtick);

        let ship_id: ShipID = ArrayString::from("ship").unwrap();
        let node = ManeuverNode {
            name: "test_node".to_string(), 
            thrust: DVec3{x: 0., y: 0., z: 0.}, 
            origin: ArrayString::from("terre").unwrap()
        };
        let ship_schedule = ShipSchedule (vec![(1, ShipActionKind::AddNode { node: node.clone() })]);
        let info = ShipInfo { 
            id: ArrayString::from("ship").unwrap(),
            ..default()
        };
        app.world_mut().spawn((info, ship_schedule));
        app.world_mut().insert_resource(GameTime { simtick: 8 });

        app.world_mut().run_schedule(FixedUpdate);
        app.update();
        let mut reader = app.world_mut().get_resource_mut::<Events<TrajectoryEvent>>().unwrap();
        let event_iter = reader.drain();
        let mut received = Vec::new();
        for event in event_iter {
            received.push(event.clone());
        }
        assert_eq!(received.len(), 0);

        app.world_mut().run_schedule(FixedUpdate);
        app.update();

        let mut reader: Mut<'_, Events<TrajectoryEvent>> = app.world_mut().get_resource_mut::<Events<TrajectoryEvent>>().unwrap();
        let event_iter = reader.drain();
        let mut received = Vec::new();
        for event in event_iter {
            received.push(event.clone());
        }
        assert_eq!(received.len(), 1);
        assert_eq!(
        received,
        vec![TrajectoryEvent::AddNode {
            ship: ship_id,
            node,
            tick: 1,
        }]);
    }
}