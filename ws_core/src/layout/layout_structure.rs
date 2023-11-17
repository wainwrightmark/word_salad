use glam::Vec2;
use crate::rect::Rect;

pub trait LayoutStructure: Sized + Clone + Copy + 'static {
    const ROOT: Self;

    fn pick(point: &Vec2) -> Option<Self> {
        if !Self::ROOT.rect().contains(point) {
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

                if child.rect().contains(point) {
                    current = child;
                    continue 'outer;
                }

                child_index += 1;
            }
        }
        Some(current)
    }

    fn rect(&self) -> Rect {
        Rect {
            top_left: self.location(),
            extents: self.size(),
        }
    }

    ///The size on a 320x568 canvas
    fn size(&self) -> Vec2;
    fn location(&self) -> Vec2;

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