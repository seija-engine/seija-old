use nalgebra::{Vector2,Vector3};
use specs::{WorldExt};
use shred::{DispatcherBuilder, World};


pub mod types;
pub mod view;
pub mod stack;
pub mod system;

use specs::{Entity,Component, DenseVecStorage, ReadStorage, WriteStorage};
pub use system::LayoutData;
pub use stack::{Stack};
pub use view::{View};
pub use types::{LayoutAlignment,Thickness};

use crate::{common::{Rect2D, Transform, TreeNode}, window::ViewPortSize};

pub fn init_layout_system(world:&mut World,builder:&mut DispatcherBuilder<'static,'static>) {
    world.register::<LayoutElement>();
    let layout_system = system::LayoutSystem::new(world);
    builder.add(layout_system, "layout", &[]);
}

pub trait IView {
  fn measure(&self,entity:Entity,size:Vector2<f64>
             ,rects:&mut WriteStorage<Rect2D>
             ,tree_nodes:&ReadStorage<TreeNode>
             ,elems:&WriteStorage<LayoutElement>) -> Vector2<f64>;
  fn arrange(&self,entity:Entity, size:Vector2<f64>
        ,rects:&mut WriteStorage<Rect2D>
        ,tree_nodes:&ReadStorage<TreeNode>
        ,elems:&WriteStorage<LayoutElement>
        ,trans:&mut WriteStorage<Transform>
        ,origin:Vector3<f32>);
}

pub enum LayoutElement {
    ViewUnit(View),
    StackLayout(Stack)
}

impl Component for LayoutElement {
    type Storage = DenseVecStorage<LayoutElement>;
}

impl IView for LayoutElement {
    fn measure(&self,entity:Entity, size:Vector2<f64>
               ,rects:&mut WriteStorage<Rect2D>
               ,tree_nodes:&ReadStorage<TreeNode>
               ,elems:&WriteStorage<LayoutElement>) -> Vector2<f64> {
        match self {
            LayoutElement::ViewUnit(v) => {
                v.measure(entity,size, rects,tree_nodes,elems)
            },
            LayoutElement::StackLayout(stack) => {
                stack.measure(entity,size, rects,tree_nodes,elems)
            }
        }
    }
    
    fn arrange(&self, entity:Entity, size:Vector2<f64>
                    , rects:&mut WriteStorage<Rect2D>
                    , tree_nodes:&ReadStorage<TreeNode>
                    , elems:&WriteStorage<LayoutElement>
                    , trans:&mut WriteStorage<Transform>
                    , origin:Vector3<f32>) {
       match self {
            LayoutElement::ViewUnit(v) => {
                v.arrange(entity,size, rects,tree_nodes,elems,trans,origin)
            },
            LayoutElement::StackLayout(stack) => {
                stack.arrange(entity,size, rects,tree_nodes,elems,trans,origin)
            }
        } 
    }
    
}

impl LayoutElement {
    fn update_layout(&self,entity:Entity,tree_nodes:&ReadStorage<TreeNode>
                              ,rects:&mut WriteStorage<Rect2D>
                              ,elems:&WriteStorage<LayoutElement>
                              ,view_size:&ViewPortSize
                              ,trans:&mut WriteStorage<Transform>
                              ) {
        let size:Vector2<f64> = self.size_request(entity,tree_nodes, rects,elems,view_size);
       
        self.measure(entity,size, rects,tree_nodes,elems);
        let origin = LayoutElement::origin_request(entity, tree_nodes, view_size, rects);
        self.arrange(entity, size, rects, tree_nodes, elems,trans,origin);
    }

    fn origin_request(entity:Entity
                      ,tree_nodes:&ReadStorage<TreeNode>
                      ,view_size:&ViewPortSize
                      ,rects:&WriteStorage<Rect2D>) -> Vector3<f32> {
        let parent = tree_nodes.get(entity).and_then(|t| t.parent);
        if let Some(p) = parent {
            let rect = rects.get(p).unwrap();
            Vector3::new(rect.left(),rect.top(),0f32)
        } else {
            let w = view_size.width() as f32;
            let h = view_size.height() as f32;
            Vector3::new(-w * 0.5f32,h * 0.5f32,0f32)
        }
        
    }
    
    fn size_request(&self,entity:Entity
                         ,tree_nodes:&ReadStorage<TreeNode>
                         ,rects:&WriteStorage<Rect2D>
                         ,elems:&WriteStorage<LayoutElement>
                         ,view_size:&ViewPortSize)  -> Vector2<f64> {
        let fsize:Vector2<f64> = self.fview(|v| v.size);
        if fsize.magnitude() > 0f64 {
            return fsize;
        }
       if let Some(parent) = tree_nodes.get(entity).and_then(|t| t.parent) {
           let elem = elems.get(parent).unwrap();
           let mut size:Vector2<f64> = elem.size_request(parent,tree_nodes, rects, elems,view_size);
           let (h,v) = elem.fview(|e| (e.padding.horizontal(),e.padding.vertical()));
           size.x -= h;
           size.y -= v;
           size
       } else {
           Vector2::new(view_size.width(),view_size.height())
       }
    }
   
    

    fn fview<F,R>(&self,f:F) -> R where F:Fn(&View) -> R {
        match self {
            LayoutElement::ViewUnit(view) => {
              f(view) 
            },
            LayoutElement::StackLayout(stack) => {
               f(&stack.view)
            }
        }
    }
}