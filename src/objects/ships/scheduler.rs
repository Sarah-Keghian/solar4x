use super::trajectory::{TrajectoryEvent, ManeuverNode};
use crate::objects::ships::{ShipID, ShipsMapping};
use crate::physics::time::GameTime;
use crate::ui::screen::editor::EditorEvents;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, handle_schedules)
        .add_systems(Update, create_schedule);
}
enum ShipActionKind {
    AddNode{ node: ManeuverNode },
    // OtherAction,
}
#[derive(Component)]
struct ShipSchedule {
    ship: ShipID,
    actions: Vec<(u64, ShipActionKind)>
}
impl ShipSchedule {
    fn new(ship: ShipID) -> Self {
        Self {
            ship,
            actions: Vec::new(),
        }
    }
}
fn handle_schedules (
    mut query: Query<&mut ShipSchedule>,
    mut traj_writer:EventWriter<TrajectoryEvent>,
    time: Res<GameTime>, 
) {
    for mut schedule in query.iter_mut() {
        let mut i: usize = 0;
        while i < schedule.actions.len() {
            if schedule.actions[i].0 <= time.tick() {
                let (tick, kind) = schedule.actions.remove(i);
                let ship = schedule.ship;
                convert_kind(tick, &kind, &ship, &mut traj_writer);
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
                                                            ship: ship.clone(),
                                                            node: node.clone(),
                                                            tick: tick,
                                                            }
                                                        );
        },
     // ShipActionKind::OtherAction => ...
    }
}

fn create_schedule(
    mut commands: Commands,
    mut events: EventReader<EditorEvents>,
    mapping: Res<ShipsMapping>,
) {
    for event in events.read() {
        if let EditorEvents::CreateSchedule(ship_id) = event {
            if let Some(ship) = mapping.0.get(ship_id) {
                commands.entity(*ship).insert(ShipSchedule::new(*ship_id));
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
    fn test_create_schedule() {
        let mut app = App::new();
        
        app.add_event::<EditorEvents>()
            .init_resource::<ShipsMapping>()
            .add_systems(Update, create_schedule);

        #[derive(Component, Clone)]
        struct ShipInfo{ id: ShipID }
        let ship_id: ShipID = ArrayString::from("ship").unwrap();
        let info = ShipInfo{ id: ship_id };
        let ship = app.world_mut().spawn(info.clone()).id();
        let mut ships_mapping = app.world_mut().resource_mut::<ShipsMapping>();
        ships_mapping.0.insert(info.id, ship);
        app.world_mut().send_event(EditorEvents::CreateSchedule(info.id));
        
        app.update();
        let schedule = app.world().get::<ShipSchedule>(ship);
        assert!(schedule.is_some(), "ShipSchedule should be inserted on ship entity");

    }

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
        let ship_schedule = ShipSchedule {
            ship: ship_id,
            actions: vec![(1, ShipActionKind::AddNode { node: node.clone() })],
        };
        app.world_mut().spawn(ship_schedule);
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
            node: node,
            tick: 1,
        }]);
    }

}