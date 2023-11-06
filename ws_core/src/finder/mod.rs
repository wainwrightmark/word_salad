pub mod node;
pub mod helpers;
pub mod partial_grid;
pub mod counter;

pub type Tile = geometrid::tile::Tile<4, 4>;
pub type NodeId = geometrid::tile::Tile<16, 1>;
pub type NodeIdSet = geometrid::tile_set::TileSet16<16,1,16>;
pub type NodeTiles = geometrid::tile_map::TileMap<Option<Tile>, 16, 1, 16>;
