pub mod entities;
pub mod flex;
pub mod layout_sizing;
pub mod layout_structure;
pub mod rect;
pub mod spacing;

pub mod prelude {

    pub use crate::layout::flex::*;
    pub use crate::layout::layout_sizing::*;
    pub use crate::layout::layout_structure::*;
    pub use crate::layout::rect::*;
    pub use crate::layout::spacing::*;
}

#[cfg(test)]
mod tests {
    use glam::Vec2;
    use strum::IntoEnumIterator;

    use crate::layout::entities::*;
    use crate::prelude::*;

    // TODO check that all children are contained within parents
    // TODO check that all siblings do not intersect each other

    #[test]
    fn test_picking_all() {
        let sizing = LayoutSizing::default();
        let selfie_mode = SelfieMode {
            is_selfie_mode: false,
        };

        test_picking::<GameLayoutEntity>(&selfie_mode, &sizing);
        test_picking::<LayoutTopBar>(&(), &sizing);
        // test_picking::<LayoutPuzzleTheme>(&());
        test_picking::<LayoutGridTile>(&selfie_mode, &sizing);
        //test_picking::<LayoutWordTile>(&()); //TODO test this
    }

    fn test_picking<T: LayoutStructure + Copy>(context: &T::Context<'_>, sizing: &LayoutSizing) {
        for entity in T::iter_all(context) {
            let rect = entity.rect(context, sizing);

            let centre_expected = T::pick(rect.centre(), context, sizing);

            assert_eq!(Some(entity), centre_expected, "Centre");
        }
    }

    #[test]
    fn svg() {
        let size = Vec2 {
            x: (IDEAL_WIDTH) as f32,
            y: (IDEAL_HEIGHT) as f32,
        };

        let layout = LayoutSizing::from_page_size(size, IDEAL_RATIO, IDEAL_WIDTH);
        let selfie_mode = SelfieMode {
            is_selfie_mode: false,
        };

        let mut svg = format!(
            r#"
        <svg version="1.1" id="Layer_1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"
            viewBox="0 0 {} {}" xml:space="preserve">
        "#,
            size.x, size.y
        );

        for layout_entity in GameLayoutEntity::iter() {
            let layout_size = layout.get_size(&layout_entity, &selfie_mode);
            let (width, height) = (layout_size.x, layout_size.y);
            let Vec2 { x, y } = layout.get_location(&layout_entity, &selfie_mode);

            let color = match layout_entity {
                GameLayoutEntity::TopBar => "blue",

                GameLayoutEntity::Theme => "coral",
                GameLayoutEntity::ThemeInfo => "gold",

                GameLayoutEntity::Grid => "indigo",

                GameLayoutEntity::WordList => "mediumblue",
                GameLayoutEntity::Timer => "pink",
            };

            let id = layout_entity.to_string();

            svg.push_str(format!(r#"<rect id="{id}" x="{x}" y="{y}" width="{width}" height="{height}" fill="{color}" opacity="0.8" />"#).as_str());
            svg.push('\n');
        }

        svg.push_str("</svg>");

        println!("{svg}");
        //TODO test with insta
    }
}
