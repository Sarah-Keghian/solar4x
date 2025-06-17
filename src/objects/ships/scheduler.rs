use super::trajectory::{TrajectoryEvent, ManeuverNode};
use crate::objects::ships::ShipID;
use crate::physics::time::GameTime;
use crate::ui::screen::editor::EditorEvents;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, handle_schedules)
        .add_systems(Update, create_schedules);
}

#[derive(Clone)]
enum ShipActionKind {
    AddNode {
        node: ManeuverNode,
        tick: u64,
    },
    OtherAction,
}

#[derive(Component, Clone)]
struct Schedule {
    ship: ShipID,
    actions: Vec<(u64, ShipActionKind)>
}

impl Schedule {
    fn new(ship: ShipID) -> Self {
        Self {
            ship,
            actions: Vec::new(),
        }
    }
}

fn handle_schedules (
    mut query: Query<&mut Schedule>,
    mut traj_writer:EventWriter<TrajectoryEvent>,
    time: Res<GameTime>, 
) {
    for mut schedule in query.iter_mut() {
        let mut i: usize = 0;
        while i < schedule.actions.len() {
            if schedule.actions[i].0 <= time.tick() {
                let (_tick, kind) = schedule.actions.remove(i);
                let ship = schedule.ship;
                convert_kind(&kind, &ship, &mut traj_writer);        
            } else {
                i += 1;
            }
        }
    }
}

fn convert_kind(
    kind: &ShipActionKind,
    ship: &ShipID,
    traj_writer: &mut EventWriter<TrajectoryEvent>
) {
    if let ShipActionKind::AddNode { node, tick } = kind {
        traj_writer.send(TrajectoryEvent::AddNode {
                        ship: ship.clone(),
                        node: node.clone(),
                        tick: *tick,
                        });
    }
}

fn create_schedules(
    mut commands: Commands,
    mut events: EventReader<EditorEvents>,
) {
    for event in events.read() {
        if let EditorEvents::CreateSchedule{ship, ship_id} = event {
            commands.entity(*ship).insert(Schedule::new(*ship_id));
        }
    }
}
