pub mod character;
pub mod constants;
pub mod paths;
pub mod state;
pub mod view;
pub mod word;
use bevy::{input::mouse::MouseButtonInput, log::LogPlugin, window::PrimaryWindow};

pub use crate::prelude::*;
fn main() {
    let mut app = App::new();

    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: "steks".to_string(),
            canvas: Some("#game".to_string()),
            resolution: bevy::window::WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            resize_constraints: WindowResizeConstraints {
                min_height: WINDOW_HEIGHT,
                min_width: WINDOW_WIDTH,
                max_width: WINDOW_WIDTH,
                max_height: WINDOW_HEIGHT,
            },
            present_mode: bevy::window::PresentMode::default(),

            resizable: true,
            ..Default::default()
        }),
        ..Default::default()
    };

    let log_plugin = LogPlugin {
        level: bevy::log::Level::INFO,
        ..Default::default()
    };

    app.insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::ALICE_BLUE))
        .add_plugins(
            DefaultPlugins
                .set(window_plugin)
                .set(log_plugin)
                .build()
                .add_before::<bevy::asset::AssetPlugin, _>(
                    bevy_embedded_assets::EmbeddedAssetPlugin,
                ),
        )
        .add_plugins(ShapePlugin)
        .add_systems(Startup, setup_system);

    app.register_maveric::<ViewRoot>();
    app.add_plugins(StatePlugin);

    app.register_transition::<FillColorLens>();

    app.add_systems(Update, on_click);
    //app.add_systems(Update, draw_shape);
    app.add_systems(Update, button_system);

    app.run();
}

fn setup_system(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn adjust_cursor_position(p: Vec2) -> Vec2 {
    Vec2 {
        x: p.x - (WINDOW_WIDTH * 0.5),
        y: (WINDOW_HEIGHT * 0.5) - p.y,
    }
}

fn on_click(
    mut events: EventReader<MouseButtonInput>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<ChosenState>,
    level: Res<CurrentLevel>,
) {
    for ev in events.into_iter() {
        if !ev.state.is_pressed() {
            continue;
        }
        if ev.button != MouseButton::Left {
            continue;
        }

        for window in q_windows.iter() {
            let Some(cursor_position) = window.cursor_position() else {
                continue;
            };
            let cursor_position = adjust_cursor_position(cursor_position);
            let tile = pick_tile(cursor_position);

            if let Some(tile) = Tile::try_from_dynamic(tile) {
                state.on_click(tile, &level);
            }
        }
    }
}

pub fn pick_tile(position: Vec2) -> DynamicTile {
    let position = position - TOP_LEFT - (TILE_SIZE * 0.5);

    let dv = DynamicVertex::from_center(&position, SCALE);
    let dt = dv.get_tile(&Corner::SouthEast);
    dt
}

fn button_system(
    mut interaction_query: Query<(&Interaction, &ButtonMarker), Changed<Interaction>>,
    mut current_level: ResMut<CurrentLevel>,
    // mut found_words: ResMut<FoundWordsState>,
    // mut chosen_state: ResMut<ChosenState>,
) {
    for (interaction, button_marker) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            match button_marker {
                ButtonMarker::Reset => {
                    current_level.set_changed();
                }
                ButtonMarker::NextLevel => {
                    *current_level = CurrentLevel::get_level(current_level.level_index + 1)
                }
            }
        }
    }
}

pub mod prelude {

    pub use crate::character::*;
    pub use crate::constants::*;
    pub use crate::state::*;
    pub use crate::view::*;
    pub use crate::word::*;
    pub use std::array;

    pub use bevy::prelude::*;
    use bevy_prototype_lyon::prelude::tess::geom::arrayvec::ArrayVec;
    pub use geometrid::prelude::*;

    pub use bevy_prototype_lyon::prelude::*;
    pub use geometrid::prelude::HasCenter;
    pub use maveric::prelude::*;

    pub type Tile = geometrid::tile::Tile<4, 4>;
    pub type CharsArray = ArrayVec<Character, 16>;
    pub type Grid = geometrid::tile_map::TileMap<Character, 4, 4, 16>;
    pub type GridSet = geometrid::tile_set::TileSet16<4, 4, 16>;
    pub type Vertex = geometrid::vertex::Vertex<4, 4>;
    pub type Solution = ArrayVec<Tile, 16>;

    pub fn try_make_grid(text: &str) -> Option<Grid> {
        let mut arr = [Character::Blank; 16];
        for (index, char) in text.chars().enumerate() {
            let c = Character::try_from(char).ok()?;
            *arr.get_mut(index)? = c;
        }

        Some(Grid::from_inner(arr))
    }
}
