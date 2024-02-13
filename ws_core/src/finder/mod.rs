use crate::CharacterSet;

pub mod counter;
pub mod helpers;
pub mod node;
pub mod orientation;
pub mod partial_grid;
pub mod cluster_ordering;
pub mod cluster;

pub type Tile = geometrid::tile::Tile<4, 4>;

pub(crate) type NodeId = geometrid::tile::Tile<16, 1>;
pub(crate) type NodeIdSet = geometrid::tile_set::TileSet16<16, 1, 16>;
pub(crate) type NodeTiles = geometrid::tile_map::TileMap<Option<Tile>, 16, 1, 16>;

pub(crate) type CharacterNodes = CharacterSet<NodeIdSet>;

/// An id of a multi-constraint - forces a node to be next to a tile with a particular character
pub(crate) type MultiConstraintId = geometrid::tile::Tile<8, 1>;

/// Set of multi-constraint ids
pub(crate) type MultiConstraintIdSet = geometrid::tile_set::TileSet8<8, 1, 8>;

/// Map from multi-constraint ids to node ids
pub(crate) type MultiConstraintMap = geometrid::tile_map::TileMap<NodeIdSet, 8, 1, 8>;

pub(crate) type CharacterMultiConstraints = CharacterSet<Option<MultiConstraintId>>;
