use crate::layout::prelude::*;
use glam::Vec2;

pub struct LayoutSizing {
    pub size_ratio: f32,
    pub left_pad: f32,
}

impl LayoutSizing {
    pub fn from_page_size(page_size: Vec2, ideal_ratio: f32, ideal_width: f32) -> Self {
        let ratio = page_size.x / page_size.y;

        //let used_y: f32;
        let used_x: f32;

        if ratio >= ideal_ratio {
            // There is additional x, so just left pad everything
            //used_y = page_size.y;
            used_x = page_size.y as f32 * ideal_ratio;
        } else {
            // There is additional y, so don't use the bottom area
            used_x = page_size.x as f32;
            //used_y = page_size.x / IDEAL_RATIO;
        }

        let left_pad = ((page_size.x as f32 - used_x) / 2.).round();
        let size_ratio = used_x / ideal_width as f32;

        Self {
            size_ratio,
            left_pad,
        }
    }

    pub fn try_pick_entity<T: LayoutStructure>(
        &self,
        position: Vec2,
        tolerance: f32,
        context: &T::Context,
    ) -> Option<T> {
        let x = position.x - self.left_pad;
        let y = position.y;

        let x = (x / self.size_ratio).round() as f32;
        let y = (y / self.size_ratio).round() as f32;

        let location = Vec2 { x, y };

        let entity = T::pick(location, context)?;

        if tolerance >= 1.0 {
            return Some(entity);
        }

        let rect: LayoutRectangle = entity.rect(context).into();

        let dist = rect.centre().distance(location);
        let size_squared = rect.extents.length();

        if dist / size_squared < tolerance {
            return Some(entity);
        }
        return None;
    }

    pub fn get_size<T: LayoutStructure>(&self, entity: &T, context: &T::Context) -> Vec2 {
        let v2: Vec2 = entity.size(context);
        v2 * self.size_ratio
    }

    pub fn get_location<T: LayoutStructure>(&self, entity: &T, context: &T::Context) -> glam::Vec2 {
        let Vec2 { x, y } = entity.location(context);

        Vec2 {
            x: self.left_pad + (self.size_ratio * x as f32),
            y: (self.size_ratio * y as f32),
        }
    }

    pub fn get_rect<T: LayoutStructure>(
        &self,
        entity: &T,
        context: &T::Context,
    ) -> LayoutRectangle {
        LayoutRectangle {
            top_left: self.get_location(entity, context),
            extents: self.get_size(entity, context),
        }
    }

    pub fn font_size<T: LayoutStructureWithFont>(&self) -> f32 {
        const FONT_INTERVAL: f32 = 4.0;
        let base_size = T::font_size();

        (self.size_ratio * base_size / FONT_INTERVAL).floor() * FONT_INTERVAL
    }
}
