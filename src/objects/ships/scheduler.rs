use super::trajectory::ManeuverNode;
use crate::objects::ships::ShipID;
use crate::physics::time::GameTime;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_event::<ShipAction>()
        .add_systems(Update, handle_schedules);
        
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
enum ShipAction {
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

#[derive(Component, Clone)] // Penser à insérer ce composant dans les entités ships
struct Scheduler {
    ship: ShipID,
    actions: Vec<(u64, ShipActionKind)>
}

fn handle_schedules (
    mut query: Query<&mut Scheduler>,
    mut writer: EventWriter<ShipAction>,
    time: Res<GameTime>, 
) {
    for mut scheduler in query.iter_mut() {
        let i: usize = 0;
        while i < scheduler.actions.len() {
            if scheduler.actions[i].0 <= time.tick() {
                let (_tick, kind) = scheduler.actions.remove(i);
                let action = ShipAction::with_ship(scheduler.ship, kind);
                writer.send(action);
            }
        }
    }
}


