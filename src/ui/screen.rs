use bevy::prelude::*;
use bevy_ratatui::{event::KeyEvent, terminal::RatatuiContext};
use editor::{EditorContext, EditorScreen};
use explorer::{ExplorerContext, ExplorerScreen};
use fleet::{FleetContext, FleetScreen};
use start::{StartMenu, StartMenuContext};

use crate::{
    client::ClientMode,
    objects::ships::ShipID,
    prelude::{exit_on_error_if_app, Loaded},
};

use super::{widget::space_map::SpaceMap, InputReading, RenderSet};

pub mod editor;
pub mod explorer;
pub mod fleet;
pub mod start;
pub mod schedule_screen;

/// A resource storing the current screen
/// Set this to change screen, the appropriate context is automatically generated when the app is ready
/// (for example when the bodies have been imported)
#[derive(States, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub enum AppScreen {
    #[default]
    StartMenu,
    Explorer,
    Fleet,
    Editor(ShipID),
    Scheduler(ShipID),
}

#[derive(Resource, Default, Debug)]
pub struct PreviousScreen(pub AppScreen);

pub fn plugin(app: &mut App) {
    app.add_plugins((
        start::plugin,
        explorer::plugin,
        fleet::plugin,
        editor::plugin,
    ))
    .init_state::<AppScreen>()
    .init_resource::<PreviousScreen>()
    .add_systems(
        PreUpdate,
        update_previous_screen.run_if(resource_changed::<NextState<AppScreen>>),
    )
    .add_systems(
        Update,
        clear_key_events
            .before(InputReading)
            .run_if(state_changed::<AppScreen>),
    )
    .add_systems(
        OnEnter(ClientMode::Explorer),
        move |mut next_screen: ResMut<NextState<AppScreen>>| next_screen.set(AppScreen::Explorer),
    )
    .add_systems(
        OnEnter(ClientMode::Singleplayer),
        move |mut next_screen: ResMut<NextState<AppScreen>>| next_screen.set(AppScreen::Fleet),
    )
    .add_systems(
        PostUpdate,
        render
            .pipe(exit_on_error_if_app)
            .run_if(resource_exists::<RatatuiContext>)
            .in_set(RenderSet),
    );
}

fn update_previous_screen(
    next: Res<NextState<AppScreen>>,
    current: Res<State<AppScreen>>,
    mut previous: ResMut<PreviousScreen>,
) {
    if let NextState::Pending(_) = next.as_ref() {
        previous.0 = *current.get();
    }
}

fn clear_key_events(mut events: ResMut<Events<KeyEvent>>) {
    events.clear();
}

fn render(
    mut ctx: ResMut<RatatuiContext>,
    screen: Res<State<AppScreen>>,
    start_menu: Option<ResMut<StartMenuContext>>,
    explorer: Option<ResMut<ExplorerContext>>,
    fleet: Option<ResMut<FleetContext>>,
    editor: Option<ResMut<EditorContext>>,
    space_map: Option<ResMut<SpaceMap>>,
) -> color_eyre::Result<()> {
    ctx.draw(|f| match screen.get() {
        AppScreen::StartMenu => {
            f.render_stateful_widget(StartMenu, f.size(), start_menu.unwrap().as_mut())
        }
        AppScreen::Explorer => {
            if let Some(mut explorer) = explorer {
                f.render_stateful_widget(
                    ExplorerScreen {
                        map: space_map.unwrap().as_mut(),
                    },
                    f.size(),
                    explorer.as_mut(),
                )
            }
        }
        AppScreen::Fleet => {
            f.render_stateful_widget(FleetScreen, f.size(), fleet.unwrap().as_mut())
        }
        AppScreen::Editor(_) => {
            f.render_stateful_widget(EditorScreen, f.size(), editor.unwrap().as_mut())
        }
        AppScreen::Scheduler(_) => {} // A REMPLIR UNE FOIS L'UI FAITE
    })?;
    Ok(())
}

/// Helper function to reduce boilerplate
pub fn in_loaded_screen<Context: Resource>(screen: AppScreen) -> impl Condition<()> {
    in_state(screen)
        .and_then(resource_exists::<Context>)
        .and_then(in_state(Loaded))
}
