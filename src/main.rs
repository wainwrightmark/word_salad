pub mod paths;
pub mod state;
pub mod view;
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
    level: Res<CurrentLevel>
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
                },
                ButtonMarker::NextLevel => *current_level = CurrentLevel::get_level(current_level.level_index + 1),
            }
        }
    }
}

pub mod prelude {

    pub use crate::state::*;
    pub use crate::view::*;

    pub use std::array;
    use std::usize;

    pub use bevy::prelude::*;
    use bevy_prototype_lyon::prelude::tess::geom::arrayvec::ArrayVec;
    pub use geometrid::prelude::*;

    pub use bevy_prototype_lyon::prelude::*;
    pub use geometrid::prelude::HasCenter;
    pub use maveric::prelude::*;

    pub type Tile = geometrid::tile::Tile<4, 4>;
    pub type Grid = geometrid::tile_map::TileMap<Character, 4, 4, 16>;
    pub type GridSet = geometrid::tile_set::TileSet16<4, 4, 16>;
    pub type Vertex = geometrid::vertex::Vertex<4, 4>;
    pub type Solution = ArrayVec<Tile, 16>;

    pub const WINDOW_WIDTH: f32 = 400f32;
    pub const WINDOW_HEIGHT: f32 = 800f32;
    pub const WINDOW_SIZE: f32 = if WINDOW_HEIGHT < WINDOW_WIDTH {
        WINDOW_HEIGHT
    } else {
        WINDOW_WIDTH
    };
    pub const SCALE: f32 = WINDOW_SIZE / 5.0;
    pub const TILE_SIZE: f32 = SCALE * TILE_MULTIPLIER;
    const TILE_MULTIPLIER: f32 = 0.9;

    pub const TOP_LEFT: Vec2 = Vec2 {
        x: (WINDOW_WIDTH * -0.5) + TILE_SIZE,
        y: (WINDOW_HEIGHT * -0.5) + TILE_SIZE,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum Character {
        Blank,
        A,
        B,
        C,
        D,
        E,
        F,
        G,
        H,
        I,
        J,
        K,
        L,
        M,
        N,
        O,
        P,
        Q,
        R,
        S,
        T,
        U,
        V,
        W,
        X,
        Y,
        Z,
    }

    impl TryFrom<char> for Character {
        type Error = ();

        fn try_from(value: char) -> Result<Self, Self::Error> {
            match value {
                '_' | ' ' => Ok(Character::Blank),
                'a' | 'A' => Ok(Character::A),
                'b' | 'B' => Ok(Character::B),
                'c' | 'C' => Ok(Character::C),
                'd' | 'D' => Ok(Character::D),
                'e' | 'E' => Ok(Character::E),
                'f' | 'F' => Ok(Character::F),
                'g' | 'G' => Ok(Character::G),
                'h' | 'H' => Ok(Character::H),
                'i' | 'I' => Ok(Character::I),
                'j' | 'J' => Ok(Character::J),
                'k' | 'K' => Ok(Character::K),
                'l' | 'L' => Ok(Character::L),
                'm' | 'M' => Ok(Character::M),
                'n' | 'N' => Ok(Character::N),
                'o' | 'O' => Ok(Character::O),
                'p' | 'P' => Ok(Character::P),
                'q' | 'Q' => Ok(Character::Q),
                'r' | 'R' => Ok(Character::R),
                's' | 'S' => Ok(Character::S),
                't' | 'T' => Ok(Character::T),
                'u' | 'U' => Ok(Character::U),
                'v' | 'V' => Ok(Character::V),
                'w' | 'W' => Ok(Character::W),
                'x' | 'X' => Ok(Character::X),
                'y' | 'Y' => Ok(Character::Y),
                'z' | 'Z' => Ok(Character::Z),
                _ => Err(()),
            }
        }
    }

    pub fn try_make_grid(text: &str) -> Option<Grid> {
        let mut arr = [Character::Blank; 16];
        for (index, char) in text.chars().enumerate() {
            let c = Character::try_from(char).ok()?;
            *arr.get_mut(index)? = c;
        }

        Some(Grid::from_inner(arr))
    }


    pub type CharsArray = ArrayVec<Character, 16>;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Word {
        pub characters: CharsArray,
        pub text: &'static str,
    }

    impl Word {
        pub fn from_static_str(text: &'static str) -> Result<Self, ()> {
            let mut characters = ArrayVec::<Character, 16>::default();

            for c in text.chars() {
                let character = Character::try_from(c)?;
                characters.try_push(character).map_err(|_| ())?;
            }

            Ok(Self { characters, text })
        }

        pub fn find_solution(&self, grid: &Grid) -> Option<Solution> {
            //TODO more efficient path if word has no duplicate letters
            let Some(first_char) = self.characters.get(0) else {
                return Default::default();
            };

            for first_tile in Tile::iter_by_row().filter(|tile| grid[*tile] == *first_char) {
                let mut path: ArrayVec<Tile, 16> = Default::default();
                let mut used_tiles: GridSet = Default::default();
                let mut indices: ArrayVec<u8, 16> = Default::default();

                let mut current_index: u8 = 0;
                let mut current_tile: Tile = first_tile;
                let mut char_to_find: Character = match self.characters.get(1) {
                    Some(c) => *c,
                    None => {
                        path.push(current_tile);
                        return Some(path);
                    }
                };

                loop {
                    if let Some(vector) = Vector::UNITS.get(current_index as usize) {
                        current_index += 1;
                        if let Some(adjacent_tile) = current_tile + vector {
                            if grid[adjacent_tile] == char_to_find {
                                if used_tiles.get_bit(&adjacent_tile) == false {
                                    //we need to go deeper
                                    path.push(current_tile);
                                    used_tiles.set_bit(&current_tile, true);
                                    indices.push(current_index);
                                    current_index = 0;
                                    current_tile = adjacent_tile;
                                    char_to_find = match self.characters.get(path.len() + 1) {
                                        Some(c) => *c,
                                        None => {
                                            path.push(current_tile);
                                            return Some(path);
                                        }
                                    };
                                }
                            }
                        }
                    } else {
                        //we have run out of options to try - go up a level
                        let Some(ct) = path.pop() else {
                            break;
                        };

                        used_tiles.set_bit(&ct, false);
                        current_tile = ct;
                        let Some(ci) = indices.pop() else {
                            break;
                        };
                        current_index = ci;

                        char_to_find = match self.characters.get(path.len() + 1) {
                            Some(c) => *c,
                            None => break,
                        };
                    }
                }
            }

            None
        }

        pub fn is_complete(&self, grid: &Grid) -> bool {
            //todo just use find_path
            let Some(first_char) = self.characters.get(0) else {
                return true;
            };

            for tile in Tile::iter_by_row().filter(|tile| grid[*tile] == *first_char) {
                let mut used = GridSet::default();
                used.set_bit(&tile, true);
                if self.is_complete_helper(grid, 1, tile, used) {
                    return true;
                }
            }
            false
        }

        fn is_complete_helper(
            &self,
            grid: &Grid,
            index: usize,
            previous: Tile,
            used: GridSet,
        ) -> bool {
            let Some(char) = self.characters.get(index) else {
                return true;
            };

            for tile in previous
                .iter_adjacent()
                .filter(|t| &grid[*t] == char)
                .filter(|t| !used.get_bit(t))
            {
                let mut new_used = used.clone();
                new_used.set_bit(&tile, true);

                if self.is_complete_helper(grid, index + 1, tile, new_used) {
                    return true;
                }
            }

            false
        }
    }
}

#[cfg(test)]
mod tests {
    use arrayvec::ArrayVec;

    use crate::prelude::*;

    #[test]
    pub fn test_find_path() {
        // SGOP
        // ELWO
        // DEMK
        // VEEU

        let grid = try_make_grid("SGOPELWODEMKVEEU").expect("Should be able to make grid");
        let eevee = Word::from_static_str("eevee").expect("Should be able to make word");

        let path = eevee
            .find_solution(&grid)
            .expect("Should be able to find a path for 'eevee'");

        let expected: ArrayVec<Tile, 16> = arrayvec::ArrayVec::from_iter(
            [
                Tile::new_const::<0, 1>(),
                Tile::new_const::<1, 2>(),
                Tile::new_const::<0, 3>(),
                Tile::new_const::<1, 3>(),
                Tile::new_const::<2, 3>(),
            ]
            .into_iter(),
        );

        assert_eq!(expected, path)
    }
}
