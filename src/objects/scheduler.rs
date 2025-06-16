use std::{
    iter::Peakable, 
    collections::{btree_map, BTreeMap}
};
use ships::trajectory::TrajectoryEvent::AddNode;

enum ShipActions {
    AddNode {
        ship: ShipID,
        node: ManeuverNode,
        tick: u64,
    },
}

#[derive(Component)] // Penser à insérer ce composant dans les entités ships
struct Scheduler {
    ship: ShipID,
    actions: Peekable<btree_map::IntoIter<u64,SHipActions>>
}

fn handle_schedules (
    query: Query<&mut Scheduler>,
    time: Res<Time>
) {
    query.par_iter_mut().for_each(|mut ship, mut schedule| {
        
    })
}
