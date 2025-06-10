use std::collections::BTreeMap;
use arrayvec::ArrayString;
use rand::Rng;

use bevy::{math::DVec3, prelude::*};
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyEventKind;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Block, List, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::{
    objects::ships::trajectory::{ManeuverNode, TrajectoryEvent}, physics::time::SIMTICKS_PER_TICK, prelude::*,
};

use super::AppScreen;
use super::editor::editor_backend::ReloadPredictions;
pub mod editor_backend;

pub fn plugin(app: &mut App) {
    app.add_plugins(editor_backend::plugin)
        .add_computed_state::<InEditor>()
        .add_event::<SelectNode>()
        .add_systems(
            Update,
            (
        
                read_input.in_set(InputReading),
                ((
                    handle_select_prediction.run_if(resource_exists::<Events<SelectObjectEvent>>),
                    handle_editor_events
                )
                    .chain(),)
                    .in_set(EventHandling),
            )
                .run_if(in_state(InEditor))
                .run_if(resource_exists::<EditorContext>),
        )
        .add_systems(OnEnter(InEditor), create_screen)
        .add_systems(OnExit(InEditor), clear_screen);
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InEditor;

impl ComputedStates for InEditor {
    type SourceStates = AppScreen;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        match sources {
            AppScreen::Editor(_) => Some(Self),
            _ => None,
        }
    }
}

#[derive(Resource)]
pub struct EditorContext {
    pub ship: Entity,
    pub ship_info: ShipInfo,
    pub pos: DVec3,
    pub speed: DVec3,
    pub simtick: u64,
    pub auto_thrust_enabled: bool,
    list_state: ListState,
    /// Each maneuver node is stored here along with the associated tick, and corresponds to a prediction.
    /// Since there is a prediction for each tick, the index of the prediction is simply the number of ticks
    /// that separate the start from the maneuver node
    nodes: BTreeMap<u64, ManeuverNode>,
    predictions: Vec<Entity>,
    /// These predictions start from a maneuver node that is currently being edited. At the end of edition,
    /// the true predictions after the node are replaced by these temporary ones
    temp_predictions: Vec<Entity>,
    /// This field stores the thrust that will be added to a node when we are editing one
    editing_data: Option<DVec3>,
}

impl EditorContext {
    pub fn new(
        ship: Entity,
        ship_info: ShipInfo,
        &Position(pos): &Position,
        &Velocity(speed): &Velocity,
        tick: u64,
    ) -> Self {
        Self {
            ship,
            ship_info,
            pos,
            speed,
            simtick: tick,
            list_state: ListState::default(),
            nodes: BTreeMap::new(),
            predictions: Vec::new(),
            temp_predictions: Vec::new(),
            editing_data: None,
            auto_thrust_enabled: false,
        }
    }

    pub fn selected_node(&self) -> Option<&ManeuverNode> {
        self.selected_entry().map(|(_, n)| n)
    }
    pub fn selected_node_mut(&mut self) -> Option<&mut ManeuverNode> {
        self.selected_entry_mut().map(|(_, n)| n)
    }

    pub fn selected_tick(&self) -> Option<u64> {
        self.selected_entry().map(|(t, _)| *t)
    }

    /// Attempts to select the node at the provided tick, returning the index if successful
    pub fn select_tick(&mut self, tick: u64) -> Option<usize> {
        self.index_of_tick(tick)
            .inspect(|&i| self.list_state.select(Some(i)))
    }

    pub fn index_of_tick(&self, tick: u64) -> Option<usize> {
        self.nodes.keys().position(|t| *t == tick)
    }

    pub fn selected_entry(&self) -> Option<(&u64, &ManeuverNode)> {
        self.list_state
            .selected()
            .and_then(|i| self.nodes.iter().nth(i))
    }

    pub fn selected_entry_mut(&mut self) -> Option<(&u64, &mut ManeuverNode)> {
        self.list_state
            .selected()
            .and_then(|i| self.nodes.iter_mut().nth(i))
    }

    fn index_of_prediction_at_simtick(&self, simtick: u64) -> usize {
        (simtick - self.simtick).max(self.simtick) as usize
    }

    fn prediction_at_simtick(&self, tick: u64) -> Option<Entity> {
        self.predictions
            .get(self.index_of_prediction_at_simtick(tick))
            .cloned()
    }

    pub fn selected_prediction_entity(&self) -> Option<Entity> {
        self.selected_tick()
            .and_then(|t| self.prediction_at_simtick(SIMTICKS_PER_TICK * t))
    }

    pub fn get_node(&self, tick: u64) -> Option<&ManeuverNode> {
        self.nodes.get(&tick)
    }

    pub fn select_or_insert(&mut self, tick: u64, default: ManeuverNode) {
        self.nodes.entry(tick).or_insert(default);
        self.select_tick(tick);
    }

    pub fn change_tick(&mut self, tick: u64, newtick: u64) {
        self.nodes
            .remove(&tick)
            .map(|val| self.nodes.insert(newtick, val));
    }
}
impl ClampedList for EditorContext {
    fn list_state(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }
}

pub struct EditorScreen;

#[allow(clippy::too_many_arguments)]
fn create_screen(
    mut commands: Commands,
    screen: Res<State<AppScreen>>,
    ships: Query<(&ShipInfo, &Position, &Velocity, &Influenced)>,
    ships_mapping: Res<ShipsMapping>,
    bodies_mapping: Res<BodiesMapping>,
    bodies: Query<&BodyInfo>,
    system_size: Res<SystemSize>,
    time: Res<GameTime>,
) {
    if let AppScreen::Editor(id) = screen.get() {
        if let Some(e) = ships_mapping.0.get(id) {
            let (
                info,
                pos,
                speed,
                Influenced {
                    main_influencer, ..
                },
            ) = ships.get(*e).unwrap();
            commands.insert_resource(EditorContext::new(
                *e,
                info.clone(),
                pos,
                speed,
                time.simtick,
            ));
            let mut map = SpaceMap::new(system_size.0, *main_influencer, *main_influencer);
            map.autoscale(&bodies_mapping.0, &bodies);
            commands.insert_resource(map);
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct ClearOnEditorExit;

fn clear_screen(mut commands: Commands, query: Query<Entity, With<ClearOnEditorExit>>) {
    commands.remove_resource::<EditorContext>();
    commands.remove_resource::<SpaceMap>();
    query.iter().for_each(|e| commands.entity(e).despawn());
}

fn handle_auto_thrust(ctx: &mut EditorContext, trajectory_event: &mut EventWriter<TrajectoryEvent>) {
    let tick_interval = 25u64;
    if ctx.auto_thrust_enabled {
        trajectory_event.send(TrajectoryEvent::RemoveAutoThrust {
            ship: ctx.ship_info.id,
            tick_interval,
        });
        ctx.auto_thrust_enabled = false;
    } else {
        let mut rng = rand::thread_rng();
        let mut node_list: Vec<ManeuverNode> = Vec::new();
        for _ in 0..10 {
            let x: f64 = rng.gen_range(200000.0..500000.0);
            let y: f64 = rng.gen_range(200000.0..500000.0);
            node_list.push(ManeuverNode{
            name: "auto_node".to_string(),
            thrust: DVec3::new(x, y, 0.0),
            origin: ArrayString::from("terre").unwrap()})
        }
        trajectory_event.send(TrajectoryEvent::AddAutoThrust { ship: ctx.ship_info.id, node_list, tick_interval });
        ctx.auto_thrust_enabled = true;
    }
}

fn read_input(
    mut key_event: EventReader<KeyEvent>,
    keymap: Res<Keymap>,
    mut internal_event: EventWriter<SelectNode>,
    mut next_screen: ResMut<NextState<AppScreen>>,
    mut trajectory_event: EventWriter<TrajectoryEvent>,
    mut ctx: ResMut<EditorContext>
) {
    use Direction2::*;
    use SelectNode::*;
    let keymap = &keymap.editor;
    for event in key_event.read() {
        if event.kind == KeyEventKind::Release {
            return;
        }
        if keymap.auto_thrust.matches(event) {
            handle_auto_thrust(&mut ctx, &mut trajectory_event);
            continue;
        }
        internal_event.send(match event {
            e if keymap.select_next.matches(e) => SelectAdjacent(Down),
            e if keymap.select_previous.matches(e) => SelectAdjacent(Up),
            e if keymap.back.matches(e) => return next_screen.set(AppScreen::Fleet),
            // e if keymap.new_node.matches(e) => NewNode(None),
            _ => return,
        });
    }
}

#[derive(Event, Clone, Copy)]
pub enum SelectNode {
    SelectAdjacent(Direction2),
    SelectNearestOrInsert(u64),
}

fn handle_editor_events(
    mut context: ResMut<EditorContext>,
    mut select_events: EventReader<SelectNode>,
    mut traj_events : EventReader<TrajectoryEvent>,
    mut reload_predictions : EventWriter<ReloadPredictions>,
    bodies: Query<&BodyInfo>,
    primary: Query<&BodyInfo, With<PrimaryBody>>,
    space_map: Res<SpaceMap>,
) {
    for event in select_events.read() {
        match *event {
            SelectNode::SelectAdjacent(d) => context.select_adjacent(d),
            SelectNode::SelectNearestOrInsert(simtick) => {
                let origin = space_map
                    .focus_body
                    .map_or(primary.single().0.id, |e| bodies.get(e).unwrap().0.id);
                context.select_or_insert(
                    simtick / SIMTICKS_PER_TICK,
                    ManeuverNode {
                        name: "Node".into(),
                        thrust: DVec3::ZERO,
                        origin,
                    },
                );
            }
        }
    }
    for event in traj_events.read() {
        match event {
            TrajectoryEvent::AddAutoThrust { node_list, tick_interval,.. } => {
                (0..10).for_each(|i| {context.select_or_insert(i* *tick_interval, node_list[i as usize].clone());});
                reload_predictions.send_default();
            }
            TrajectoryEvent::RemoveAutoThrust {tick_interval, ..} => {
                (0..10).for_each(|i| {
                    let key = i*tick_interval;
                    context.nodes.remove(&key);});
                reload_predictions.send_default();
            }
            _ => {},
        }
    }
}

fn handle_select_prediction(
    mut select_events: EventReader<SelectObjectEvent>,
    mut editor_events: EventWriter<SelectNode>,
    predictions: Query<&Prediction>,
) {
    for event in select_events.read() {
        if let Ok(p) = predictions.get(event.entity) {
            editor_events.send(SelectNode::SelectNearestOrInsert(p.simtick));
        }
    }
}

impl StatefulWidget for EditorScreen {
    type State = EditorContext;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let chunks =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Fill(1)]).split(area);
        let list = List::new(state.nodes.values().map(|n| &n.name[..]))
            .highlight_symbol(">")
            .block(Block::bordered().title_top("Maneuver nodes"));
        StatefulWidget::render(list, chunks[0], buf, &mut state.list_state);

        if let Some((tick, node)) = state.selected_entry() {
            Paragraph::new(format!(
                "Tick: {}\nThrust: {}\nOrigin: {}",
                tick, node.thrust, node.origin
            ))
            .render(chunks[1], buf);
        }
    }
}
