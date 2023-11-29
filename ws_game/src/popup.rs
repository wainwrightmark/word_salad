use crate::prelude::*;

#[derive(Debug, Clone, Copy, Resource, MavericContext, PartialEq, Eq, Default)]
pub enum PopupState {
    #[default]
    None,
    BuyMoreHints,
}
