use crate::common::{Rect2D, Transform, TreeNode};
use std::sync::RwLock;
use std::cell::Cell;
use super::{IView, GridCell,LayoutElement, types::{Thickness,LayoutAlignment}};
use nalgebra::{Vector2,Vector3};
use specs::{Component, DenseVecStorage, Entity, ReadStorage, WriteStorage};

#[derive(Default)]
pub struct View {
    pub pos:Cell<Vector3<f32>>,
    pub size:Cell<Vector2<f64>>,
    pub margin:Thickness,
    pub padding:Thickness,
    pub hor:LayoutAlignment,
    pub ver:LayoutAlignment,   
}

unsafe impl Sync for View {}

impl Component for View {
    type Storage = DenseVecStorage<View>;
}

impl View {
    pub fn calc_size(&self,size:Vector2<f64>) -> Vector2<f64> {
        let mut ret_size:Vector2<f64> = self.size.get();
       
        if ret_size.y <= 0f64 && self.hor == LayoutAlignment::Fill {
            ret_size.x = size.x - self.margin.horizontal();
        }
        if ret_size.y <= 0f64 && self.ver == LayoutAlignment::Fill {
            ret_size.y = size.y - self.margin.vertical();
        }
        
        ret_size
    }

    pub fn calc_content_size(&self,size:Vector2<f64>) -> Vector2<f64> {
        let  size = self.calc_size(size);
        
       
        size
    }

    pub fn calc_orign(&self,entity:Entity,rects:&WriteStorage<Rect2D>) -> Vector3<f32> {
        let rect = rects.get(entity).unwrap();
        Vector3::new(rect.left(),rect.top(),0f32)
    }
}

impl IView for View {
    fn measure(&self,entity:Entity, size:Vector2<f64>
               ,rects:&mut WriteStorage<Rect2D>
               ,_tree_nodes:&ReadStorage<TreeNode>
               ,_elems:&WriteStorage<LayoutElement>
               ,_cells:&ReadStorage<GridCell>) -> Vector2<f64> {
      
      let content_size:Vector2<f64> = self.calc_content_size(size);
     
      rects.get_mut(entity).map(|rect| {
        rect.width = content_size.x as f32;
        rect.height = content_size.y as f32;
      });
      
      content_size
    }

    

    fn arrange(&self, entity:Entity, _:Vector2<f64>
        , rect2ds:&mut WriteStorage<Rect2D>
        , _:&ReadStorage<TreeNode>
        , _:&WriteStorage<LayoutElement>
        ,trans:&mut WriteStorage<Transform>
        ,origin:Vector3<f32>
        ,_cells:&ReadStorage<GridCell>) {
        let (x,y,z) = {
            let pos:Vector3<f32> = self.pos.get();
            (pos.x,pos.y,pos.z)
        };
        let rect = rect2ds.get(entity).unwrap();
        let [ax,ay] = rect.anchor;
        let offset_w = rect.width  * ax;
        let offset_h = rect.height * ay;
        let new_pos:Vector3<f32> = Vector3::new(
            origin.x + offset_w + x + self.margin.left as f32,
            origin.y - offset_h + y - self.margin.top as f32,
            origin.z + z
        );
        
        trans.get_mut(entity).unwrap().set_position(new_pos);
    }
}
/*
impl LayoutView {
    pub fn measure(&self,entity:Entity,size:Vector2<f64>,
                   views:&WriteStorage<LayoutView>,
                   rect2ds:&mut WriteStorage<Rect2D>) -> Vector2<f64> {
        let view = views.get(entity).unwrap();
        let rect2d = rect2ds.get(entity).unwrap();
        let mut w:f64 = rect2d.width as f64;
        let mut h:f64 = rect2d.height as f64;
        if view.hor == LayoutAlignment::Fill {
            w = size.x - self.margin.horizontal();
        }
        if view.ver == LayoutAlignment::Fill {
            h = size.y - self.margin.vertical();
        }

        let rect2d = rect2ds.get_mut(entity).unwrap();
        rect2d.width = w as f32;
        rect2d.height = h as f32;
        Vector2::new(w,h)
    }
}*/