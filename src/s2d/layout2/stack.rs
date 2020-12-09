use specs::{Component, DenseVecStorage, Entity, ReadStorage, WriteStorage};
use nalgebra::Vector2;
use crate::common::{Rect2D, TreeNode};

use super::{LayoutData, LayoutView};



impl Component for StackLayout {
    type Storage = DenseVecStorage<StackLayout>;
}

#[derive(Default)]
pub struct StackLayout {
    
}

impl StackLayout {
    pub fn measure(&self,entity:Entity,size:Vector2<f64>,
                    rect2d:&mut WriteStorage<Rect2D>,
                    views:&WriteStorage<LayoutView>,
                    tree_nodes:&ReadStorage<TreeNode>) -> Vector2<f64> {
        let mut self_size = {
            let rect2d = rect2d.get(entity).unwrap();
            Vector2::new(rect2d.width as f64,rect2d.height as f64)
        };
        if let Some(view) = views.get(entity) {
            self_size = view.measure(entity, size,views,rect2d);
        }
        let mut sub_size = self_size;
        let childs = tree_nodes.get(entity).unwrap().children.clone();
        for child in childs.iter() {
            let child_size:Vector2<f64> =  {
                let view = views.get(*child).unwrap();
                view.measure(*child, sub_size,views,rect2d)
            };
            sub_size.y -= child_size.y;
        }
        
        let self_rect = rect2d.get_mut(entity).unwrap();
        self_rect.width = self_size.x as f32;
        self_rect.height = self_size.y  as f32;
        self_size
    }
}