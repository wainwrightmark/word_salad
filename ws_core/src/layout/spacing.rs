use glam::Vec2;

pub fn tile_offset<const WIDTH: u8, const HEIGHT: u8>(
    tile: geometrid::tile::Tile<WIDTH, HEIGHT>,
    h_spacing: Spacing,
    v_spacing: Spacing,
    full_size: Vec2,
    tile_size: Vec2,
) -> Vec2 {
    let x = h_spacing.apply(full_size.x, tile_size.x, WIDTH as usize, tile.x() as usize);
    let y = v_spacing.apply(full_size.y, tile_size.y, HEIGHT as usize, tile.y() as usize);

    Vec2 { x, y }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spacing {
    SpaceBetween,
    SpaceAround,
    Centre,
}

impl Spacing {
    pub fn apply(
        &self,
        parent_ideal_length: f32,
        child_ideal_length: f32,
        num_children: usize,
        child_index: usize,
    ) -> f32 {
        let total_padding = parent_ideal_length - (num_children as f32 * child_ideal_length);

        match self {
            Spacing::SpaceBetween => {
                if num_children == 0 {
                    return 0.;
                } else if num_children == 1 {
                    return total_padding / 2.;
                } else {
                    let padding_between_children =
                        total_padding / num_children.saturating_sub(1) as f32;
                    (padding_between_children + child_ideal_length) * child_index as f32
                }
            }
            Spacing::SpaceAround => {
                if num_children == 0 {
                    return 0.;
                } else {
                    let left_or_right_padding = total_padding / (num_children as f32 * 2.);

                    let paddings = (1 + (child_index * 2)) as f32;

                    (paddings * left_or_right_padding) + (child_index as f32 * child_ideal_length)
                }
            }
            Spacing::Centre => {
                let top_padding = total_padding / 2.;
                top_padding + (child_index as f32 * child_ideal_length)
            }
        }
    }
}
