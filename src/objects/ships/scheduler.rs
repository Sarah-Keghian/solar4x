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
    AddNode { node: ManeuverNode },
    OtherAction,
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
    if let ShipActionKind::AddNode { node } = kind {
        traj_writer.send(TrajectoryEvent::AddNode {
                        ship: ship.clone(),
                        node: node.clone(),
                        tick: tick,
                        });
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

}