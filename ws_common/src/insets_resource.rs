use maveric::helpers::{MavericContext, Resource};
use ws_core::insets::*;

#[derive(Debug, Clone, Default, PartialEq, Resource, MavericContext)]
pub struct InsetsResource(pub Insets);
