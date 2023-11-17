use glam::Vec2;
use crate::rect::Rect;

pub trait LayoutStructure: Sized + Clone + Copy + 'static {
    type Context;

    const ROOT: Self;

    fn pick(point: &Vec2, context: &Self::Context) -> Option<Self> {
        if !Self::ROOT.rect(context).contains(point) {
            return None;
        }

        let mut current = Self::ROOT;

        'outer: loop {
            let children = current.children();
            let mut child_index = 0;
            loop {
                if child_index >= children.len() {
                    break 'outer;
                }
                let child = children[child_index];

                if child.rect(context).contains(point) {
                    current = child;
                    continue 'outer;
                }

                child_index += 1;
            }
        }
        Some(current)
    }

    fn rect(&self, context: &Self::Context) -> Rect {
        Rect {
            top_left: self.location(context),
            extents: self.size(context),
        }
    }

    ///The size on a 320x568 canvas
    fn size(&self, context: &Self::Context) -> Vec2;
    fn location(&self, context: &Self::Context) -> Vec2;

    fn is_visible(&self, context: &Self::Context)-> bool;

    fn all() -> Vec<Self> {
        let mut vec = vec![Self::ROOT];

        let mut index = 0;

        while let Some(node) = vec.get(index) {
            let node = node.clone();
            vec.extend(node.children());
            index += 1;
        }
        vec
    }

    fn children(self) -> &'static [Self];
}