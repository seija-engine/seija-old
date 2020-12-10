use crate::common::{Rect2D, Transform, TreeNode};
use nalgebra::Vector2;
use specs::{Component, DenseVecStorage, Entity, ReadStorage, WriteStorage};

use super::{LayoutView,LayoutAlignment};

impl Component for StackLayout {
    type Storage = DenseVecStorage<StackLayout>;
}

pub enum Orientation {
    Horizontal,
    Vertical,
}
impl Default for Orientation {
    fn default() -> Self {
        Orientation::Horizontal
    }
}

#[derive(Default)]
pub struct StackLayout {
   pub orientation: Orientation,
   pub specing:f32
}

impl StackLayout {
    pub fn measure(
        &self,
        entity: Entity,
        size: Vector2<f64>,
        rect2d: &mut WriteStorage<Rect2D>,
        views: &mut WriteStorage<LayoutView>,
        tree_nodes: &ReadStorage<TreeNode>,
    ) -> Vector2<f64> {
        let mut self_size = {
            let rect2d = rect2d.get(entity).unwrap();
            Vector2::new(rect2d.width as f64, rect2d.height as f64)
        };
        if let Some(view) = views.get(entity) {
            self_size = view.measure(entity, size, views, rect2d);
        }
        let mut sub_size = self_size;
        let childs = tree_nodes.get(entity).unwrap().children.clone();
        for child in childs.iter() {
            let view = views.get(*child).unwrap();
            let old_size = rect2d.get_mut(*child).unwrap();
            match self.orientation {
                Orientation::Horizontal => {
                    sub_size.x = old_size.width as f64;
                }
                Orientation::Vertical => {
                    sub_size.y = old_size.height as f64;
                }
            }
            view.measure(*child, sub_size, views, rect2d);
        }

        let self_rect = rect2d.get_mut(entity).unwrap();
        self_rect.height = self_size.y as f32;
        self_rect.width = self_size.x as f32;

        self_size
    }

    pub fn arrange(
        &self,
        entity: Entity,
        rect2d: &mut WriteStorage<Rect2D>,
        tree_nodes: &ReadStorage<TreeNode>,
        trans: &mut WriteStorage<Transform>,
        views: &WriteStorage<LayoutView>,
    ) {
        let [lx, rx, by, ty] = rect2d.get(entity).unwrap().corner_point();

        let childs = tree_nodes
            .get(entity)
            .map(|v| v.children.clone())
            .unwrap_or(vec![]);
        match self.orientation {
            Orientation::Horizontal => {
                let mut x = lx;
                for child in childs {
                    let child_w = rect2d.get(child).map(|r| r.width).unwrap_or(0f32);
                    let ct = trans.get_mut(child).unwrap();
                    let lx = rect2d.get(child).unwrap().left().abs();
                    ct.set_position_x(x + lx);
                    match views.get(child).unwrap().ver {
                        LayoutAlignment::Start => {
                            ct.set_position_y(ty);
                        },
                        LayoutAlignment::Fill => {
                            
                        },
                        LayoutAlignment::End => {
                            ct.set_position_y(by);
                        },
                        LayoutAlignment::Center => {
                            
                        }
                    }
                    x += child_w + self.specing;
                }
            }
            Orientation::Vertical => {}
        }
    }

   
}
