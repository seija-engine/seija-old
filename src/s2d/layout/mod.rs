use nalgebra::{Vector2,Vector3};
use specs::{WorldExt};
use shred::{DispatcherBuilder, World};

mod grid;
pub mod types;
pub mod view;
pub mod stack;
pub mod system;

use specs::{Entity,Component, DenseVecStorage, ReadStorage, WriteStorage,FlaggedStorage};
pub use system::LayoutData;
pub use stack::{Stack,Orientation};
pub use view::{View,ContentView};
pub use types::{LayoutAlignment,Thickness,LNumber};
pub use grid::{Grid,GridCell};

use crate::{common::{Rect2D, Transform, TreeNode}, window::ViewPortSize};

pub fn init_layout_system(world:&mut World,builder:&mut DispatcherBuilder<'static,'static>) {
    world.register::<LayoutElement>();
    world.register::<GridCell>();
    let layout_system = system::LayoutSystem::new(world);
    builder.add(layout_system, "layout", &[]);
}

pub trait IView {
  fn measure(&self,entity:Entity,size:Vector2<f64>
             ,rects:&mut WriteStorage<Rect2D>
             ,tree_nodes:&ReadStorage<TreeNode>
             ,elems:&WriteStorage<LayoutElement>
             ,cells:&ReadStorage<GridCell>) -> Vector2<f64>;
  fn arrange(&self,entity:Entity, size:Vector2<f64>
        ,rects:&mut WriteStorage<Rect2D>
        ,tree_nodes:&ReadStorage<TreeNode>
        ,elems:&WriteStorage<LayoutElement>
        ,trans:&mut WriteStorage<Transform>
        ,origin:Vector3<f32>
        ,cells:&ReadStorage<GridCell>);
}

pub enum LayoutElement {
    View(View),
    ContentView(ContentView),
    StackLayout(Stack),
    GridLayout(Grid)
}

impl Component for LayoutElement {
    type Storage = FlaggedStorage<Self,DenseVecStorage<LayoutElement>>;
}

impl IView for LayoutElement {
    fn measure(&self,entity:Entity, size:Vector2<f64>
               ,rects:&mut WriteStorage<Rect2D>
               ,tree_nodes:&ReadStorage<TreeNode>
               ,elems:&WriteStorage<LayoutElement>
               ,cells:&ReadStorage<GridCell>) -> Vector2<f64> {
        match self {
            LayoutElement::View(v) => v.measure(entity,size, rects,tree_nodes,elems,cells),
            LayoutElement::StackLayout(stack) => stack.measure(entity,size, rects,tree_nodes,elems,cells),
            LayoutElement::GridLayout(grid) => grid.measure(entity, size, rects, tree_nodes, elems,cells),
            LayoutElement::ContentView(content_view) => content_view.measure(entity, size, rects, tree_nodes, elems, cells)
        }
    }
    
    fn arrange(&self, entity:Entity, size:Vector2<f64>
                    , rects:&mut WriteStorage<Rect2D>
                    , tree_nodes:&ReadStorage<TreeNode>
                    , elems:&WriteStorage<LayoutElement>
                    , trans:&mut WriteStorage<Transform>
                    , origin:Vector3<f32>
                    , cells:&ReadStorage<GridCell>) {
       match self {
            LayoutElement::View(v) => v.arrange(entity,size, rects,tree_nodes,elems,trans,origin,cells),
            LayoutElement::StackLayout(stack) => stack.arrange(entity,size, rects,tree_nodes,elems,trans,origin,cells),
            LayoutElement::GridLayout(grid) => grid.arrange(entity, size, rects, tree_nodes, elems, trans, origin,cells),
            LayoutElement::ContentView(content_view) => content_view.arrange(entity, size, rects, tree_nodes, elems, trans, origin, cells)
        } 
    }
    
}

impl LayoutElement {
    fn update_layout(&self,entity:Entity,tree_nodes:&ReadStorage<TreeNode>
                              ,rects:&mut WriteStorage<Rect2D>
                              ,elems:&WriteStorage<LayoutElement>
                              ,view_size:&ViewPortSize
                              ,trans:&mut WriteStorage<Transform>
                              ,cells:&ReadStorage<GridCell>
                              ) {
        let size_x = self.size_request_x(entity,tree_nodes, rects,elems,view_size);
        let size_y = self.size_request_y(entity,tree_nodes, rects,elems,view_size);
        let req_size:Vector2<f64> = Vector2::new(size_x,size_y);
        
        self.measure(entity,req_size, rects,tree_nodes,elems,cells);
        let origin:Vector3<f32> = LayoutElement::origin_request(entity, tree_nodes, view_size, rects);
        self.arrange(entity, req_size, rects, tree_nodes, elems,trans,origin,cells);
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
    
    fn size_request_x(&self,entity:Entity
                         ,tree_nodes:&ReadStorage<TreeNode>
                         ,rects:&WriteStorage<Rect2D>
                         ,elems:&WriteStorage<LayoutElement>
                         ,view_size:&ViewPortSize
                         )  -> f64 {
        let fsize:Vector2<f64> = self.fview(|v| v.get_size(rects.get(entity).unwrap()));
        if fsize.x > 0f64 {
            let mh= self.fview(|v| v.margin.horizontal());
            return fsize.x + mh;
        }
        if let Some(parent) = tree_nodes.get(entity).and_then(|t| t.parent) {
            let elem = elems.get(parent).unwrap();
            let size_x = elem.size_request_x(parent,tree_nodes, rects, elems,view_size);
            let h = elem.fview(|e|e.padding.horizontal());
            size_x - h
        } else {
            view_size.width()
        }
    }

    fn size_request_y(&self,entity:Entity
                         ,tree_nodes:&ReadStorage<TreeNode>
                         ,rects:&WriteStorage<Rect2D>
                         ,elems:&WriteStorage<LayoutElement>
                         ,view_size:&ViewPortSize
                         )  -> f64 {
        let  fsize:Vector2<f64> = self.fview(|v|v.get_size(rects.get(entity).unwrap()));
        if fsize.y > 0f64 {
            let mv= self.fview(|v| v.margin.vertical());
            return fsize.y + mv;
        }
        if let Some(parent) = tree_nodes.get(entity).and_then(|t| t.parent) {
            let elem = elems.get(parent).unwrap();
            let  size_y = elem.size_request_y(parent,tree_nodes, rects, elems,view_size);
            let v = elem.fview(|e|e.padding.vertical());
            size_y - v
        } else {
            view_size.height()
        }
    }
   
    

    pub fn fview<F,R>(&self,f:F) -> R where F:Fn(&View) -> R {
        match self {
            LayoutElement::View(view) => f(view) ,
            LayoutElement::StackLayout(stack) => f(&stack.view),
            LayoutElement::GridLayout(grid) => f(&grid.view),
            LayoutElement::ContentView(content_view) => f(&content_view.view)
        }
    }

    pub fn fview_mut<F,R>(&mut self,f:F) -> R where F:Fn(&mut View) -> R {
        match self {
            LayoutElement::View(view) => f(view) ,
            LayoutElement::StackLayout(stack) => f(&mut stack.view),
            LayoutElement::GridLayout(grid) => f(&mut grid.view),
            LayoutElement::ContentView(context_view) => f(&mut context_view.view)
        }
    }
}