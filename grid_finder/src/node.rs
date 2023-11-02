// use itertools::Itertools;
// use ws_core::{Character, GridSet};
// pub type Tile = geometrid::tile::Tile<4, 4>;

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct RootNodeId(usize);

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct ConstraintNodeId(usize);

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct RootNodeGroupId(usize);

// #[derive(Debug, PartialEq)]
// pub struct RootNode {
//     pub id: RootNodeId,
//     pub char: Character,
//     pub group: RootNodeGroupId,
// }

// impl RootNode {

//     pub fn find_possible_locations(){

//     }


//     pub fn can_play(&self, context: &Context, tile: &Tile) -> bool {
//         let group = context.get_group(&self.group);

//         if group.root_nodes.len() == 1 {
//             let required_adjacent_nodes = group
//                 .constraint_nodes
//                 .iter()
//                 .map(|n| context.get_constraint(n))
//                 .flat_map(|n| n.adjacent_nodes.iter())
//                 .map(|n| context.get_constraint(n))
//                 .map(|x| x.group)
//                 .unique()
//                 .count();

//             if required_adjacent_nodes > 3{
//                 let neighbours = tile.iter_adjacent().count(); //todo fast count neighbours
//                 if neighbours < required_adjacent_nodes{
//                     return false;
//                 }
//             }
//         }

//         //todo do something else here?
//         return true;
//     }
// }

// #[derive(Debug, PartialEq)]
// pub struct ConstraintNode {
//     pub id: ConstraintNodeId,
//     pub char: Character,
//     pub group: RootNodeGroupId,

//     pub adjacent_nodes: Vec<ConstraintNodeId>,
//     pub exclusive_nodes: Vec<ConstraintNodeId>,
// }

// #[derive(Debug, PartialEq)]
// pub struct RootNodeGroup {
//     pub id: RootNodeGroupId,
//     pub char: Character,
//     pub root_nodes: Vec<RootNodeId>,
//     pub constraint_nodes: Vec<ConstraintNodeId>,
// }

// impl RootNodeGroup {
//     pub fn constraint_score(&self, context: &Context) -> f32 {
//         let c_nodes = self
//             .constraint_nodes
//             .iter()
//             .flat_map(|cn| context.get_constraint(cn).adjacent_nodes.iter())
//             .unique()
//             .count() as f32;

//         c_nodes / self.root_nodes.len() as f32
//     }
// }

// #[derive(Debug, Default)]
// pub struct Context {
//     pub roots: Vec<RootNode>,
//     pub constraints: Vec<ConstraintNode>,
//     pub groups: Vec<RootNodeGroup>,
// }

// impl Context {
//     pub fn get_root_mut(&mut self, id: &RootNodeId) -> &mut RootNode {
//         self.roots.get_mut(id.0).unwrap()
//     }

//     pub fn get_group_mut(&mut self, id: &RootNodeGroupId) -> &mut RootNodeGroup {
//         self.groups.get_mut(id.0).unwrap()
//     }

//     pub fn get_constraint_mut(&mut self, id: &ConstraintNodeId) -> &mut ConstraintNode {
//         self.constraints.get_mut(id.0).unwrap()
//     }

//     pub fn get_root(&self, id: &RootNodeId) -> &RootNode {
//         self.roots.get(id.0).unwrap()
//     }

//     pub fn get_group(&self, id: &RootNodeGroupId) -> &RootNodeGroup {
//         self.groups.get(id.0).unwrap()
//     }

//     pub fn get_constraint(&self, id: &ConstraintNodeId) -> &ConstraintNode {
//         self.constraints.get(id.0).unwrap()
//     }
// }

// pub enum Node{
//     Root(RootNode),
//     Constraint(ConstraintNode)
// }

// pub struct PartialGrid{
//     pub dict:  geometrid::tile_map::TileMap<Vec<Node>, 4, 4, 16>,
//     pub unused: GridSet,
//     //pub node_locations:
// }