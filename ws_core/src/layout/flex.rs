use glam::Vec2;

use crate::{LayoutRectangle, LayoutStructure, LayoutSizing};

pub enum FlexLayout {
    Row, //todo column, row reverse etc
}

impl FlexLayout {
    pub fn try_pick<T: LayoutStructure>(
        &self,
        full_size: Vec2,
        point: Vec2,
        context: &T::Context<'_>,
        main_axis_padding: f32,
        cross_axis_padding: f32,
        sizing: &LayoutSizing
    ) -> Option<T> {
        let mut prev_rows_total_height: f32 = 0.0;
        let mut current_row_total_width: f32 = 0.0;
        let mut current_row_max_height: f32 = 0.0;

        for entity in T::iter_all(context) {
            let size = entity.size(context, sizing);

            if current_row_total_width + size.x > full_size.x {
                //new row
                prev_rows_total_height += current_row_max_height + cross_axis_padding;
                current_row_max_height = size.y;
                current_row_total_width = 0.0;
            } else {
                //continue current row
                current_row_max_height = current_row_max_height.max(size.y);
            }
            let rect = LayoutRectangle {
                top_left: Vec2 {
                    x: current_row_total_width,
                    y: prev_rows_total_height,
                },
                extents: size,
            };
            if rect.contains(point) {
                return Some(entity);
            }

            //add to current row
            current_row_total_width += size.x + main_axis_padding;
        }
        None
    }

    pub fn get_location<T: LayoutStructure>(
        &self,
        full_size: Vec2,
        to_find: &T,
        context: &T::Context<'_>,
        main_axis_padding: f32,
        cross_axis_padding: f32,
        sizing: &LayoutSizing
    ) -> Vec2 {
        let mut prev_rows_total_height: f32 = 0.0;
        let mut current_row_total_width: f32 = 0.0;
        let mut current_row_max_height: f32 = 0.0;

        for entity in T::iter_all(context) {
            let size = entity.size(context, sizing);

            if current_row_total_width + size.x > full_size.x {
                //new row
                prev_rows_total_height += current_row_max_height + cross_axis_padding;
                current_row_max_height = size.y;
                current_row_total_width = 0.0;
            } else {
                //continue current row
                current_row_max_height = current_row_max_height.max(size.y);
            }
            if entity.eq(to_find) {
                break;
            }

            //add to current row
            current_row_total_width += size.x + main_axis_padding;
        }

        Vec2 {
            x: current_row_total_width,
            y: prev_rows_total_height,
        }
    }
}
