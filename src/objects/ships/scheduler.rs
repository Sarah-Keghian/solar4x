use super::trajectory::ManeuverNode;
use crate::objects::ships::ShipID;
use crate::physics::time::GameTime;
use crate::ui::screen::editor::EditorEvents;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_event::<ShipAction>()
        .add_systems(FixedUpdate, handle_schedules)
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

#[derive(Event)]
pub enum ShipAction {
    AddNode {
        ship: ShipID,
        node: ManeuverNode,
        tick: u64,
    },
    OtherAction { ship: ShipID },
}

impl ShipAction {
    fn with_ship(ship: ShipID, kind: ShipActionKind) -> Self {
        match kind {
            ShipActionKind::AddNode { node, tick } => ShipAction::AddNode { ship, node, tick },
            ShipActionKind::OtherAction => ShipAction::OtherAction { ship },
        }
    }
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
    mut writer: EventWriter<ShipAction>,
    time: Res<GameTime>, 
) {
    for mut schedule in query.iter_mut() {
        let i: usize = 0;
        while i < schedule.actions.len() {
            if schedule.actions[i].0 <= time.tick() {
                let (_tick, kind) = schedule.actions.remove(i);
                let action = ShipAction::with_ship(schedule.ship, kind);
                writer.send(action);
            }
        }
    }
}

fn create_schedules(
    mut commands: Commands,
    mut events: EventReader<EditorEvents>,
) {
    for event in events.read() {
        if let EditorEvents::CreateSchedule{ship, ship_id} = event {
            commands.entity(*ship).insert(schedule::new(*ship_id));
        }
    }
}

fn add_action() {

}