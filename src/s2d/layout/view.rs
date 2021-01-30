use super::{
    types::{LayoutAlignment, Thickness},
    GridCell, IView, LayoutElement,
};
use crate::common::{Rect2D, Transform, TreeNode};
use nalgebra::{Vector2, Vector3};
use specs::{Component, DenseVecStorage, Entity, ReadStorage, WriteStorage};
use std::cell::Cell;

#[derive(Clone,Copy)]
pub enum ViewType {
    Static,
    Absolute
}
impl Default for ViewType {
    fn default() -> Self {
        ViewType::Static
    }
}

impl From<u32> for ViewType {
    fn from(typ: u32) -> ViewType {
        match typ {
            0 => ViewType::Static,
            1 => ViewType::Absolute,
            _ => ViewType::Absolute
        }
    }
}

impl ViewType {
    pub fn is_absolute(&self) -> bool {
        match self {
            ViewType::Static => false,
            ViewType::Absolute => true
        }
    }
    pub fn is_static(&self) -> bool {
        match self {
            ViewType::Static => true,
            ViewType::Absolute => false
        }
    }
}

#[derive(Default)]
pub struct View {
    pub pos: Cell<Vector2<f32>>,
    pub size: Cell<Vector2<f64>>,
    pub margin: Thickness,
    pub padding: Thickness,
    pub hor: LayoutAlignment,
    pub ver: LayoutAlignment,
    pub view_type:ViewType,
    pub use_rect_size:bool
}

unsafe impl Sync for View {}

impl Component for View {
    type Storage = DenseVecStorage<View>;
}

impl View {
    
    pub fn calc_content_size(&self, size: Vector2<f64>,rect:&Rect2D) -> Vector2<f64> {
        let mut ret_size: Vector2<f64> = self.get_size(rect);

        if ret_size.x <= 0f64 && self.hor == LayoutAlignment::Fill {
            ret_size.x = size.x - self.margin.horizontal();
        }
        if ret_size.y <= 0f64 && self.ver == LayoutAlignment::Fill {
            ret_size.y = size.y - self.margin.vertical();
        }

        ret_size
    }

    pub fn calc_orign(&self, entity: Entity, rects: &WriteStorage<Rect2D>) -> Vector3<f32> {
        let rect = rects.get(entity).unwrap();
        Vector3::new(rect.left(), rect.top(), 0f32)
    }

    pub fn get_size(&self,rect:&Rect2D) -> Vector2<f64> {
        if self.use_rect_size {
            Vector2::new(rect.width() as f64,rect.height() as f64)
        } else {
            self.size.get()
        }
    }
}

impl IView for View {
    fn measure(
        &self,
        entity: Entity,
        size: Vector2<f64>,
        rects: &mut WriteStorage<Rect2D>,
        _tree_nodes: &ReadStorage<TreeNode>,
        _elems: &WriteStorage<LayoutElement>,
        _cells: &ReadStorage<GridCell>,
    ) -> Vector2<f64> {
        let content_size: Vector2<f64> = self.calc_content_size(size,rects.get(entity).unwrap());
        
        rects.get_mut(entity).map(|rect| {
            rect.set_width(content_size.x as f32);
            rect.set_height(content_size.y as f32);
            
        });
       
        content_size
    }

    fn arrange(
        &self,
        entity: Entity,
        _: Vector2<f64>,
        rect2ds: &mut WriteStorage<Rect2D>,
        _: &ReadStorage<TreeNode>,
        _: &WriteStorage<LayoutElement>,
        trans: &mut WriteStorage<Transform>,
        origin: Vector3<f32>,
        _cells: &ReadStorage<GridCell>,
    ) {
        let (x, y) = {
            let pos: Vector2<f32> = self.pos.get();
            (pos.x, pos.y)
        };
        let rect = rect2ds.get(entity).unwrap();
        let [ax, ay] = rect.anchor();
        let offset_w = rect.width() * ax;
        let offset_h = rect.height() * ay;
       
        let new_x = origin.x + offset_w + x + self.margin.left as f32;
        let new_y = origin.y - offset_h + y - self.margin.top as f32;

        trans.get_mut(entity).unwrap().set_position_xy(new_x,new_y);
    }
}

#[derive(Default)]
pub struct ContentView {
    pub view: View,
}

unsafe impl Sync for ContentView {}

impl Component for ContentView {
    type Storage = DenseVecStorage<ContentView>;
}

impl IView for ContentView {
    fn measure(
        &self,
        entity: Entity,
        size: Vector2<f64>,
        rects: &mut WriteStorage<Rect2D>,
        tree_nodes: &ReadStorage<TreeNode>,
        elems: &WriteStorage<LayoutElement>,
        cells: &ReadStorage<GridCell>,
    ) -> Vector2<f64> {
        let mut content_size:Vector2<f64> = self.view.measure(entity, size, rects, tree_nodes, elems, cells);
        let inner_size:Vector2<f64> = Vector2::new(content_size.x - self.view.padding.horizontal(),
                                                   content_size.y - self.view.padding.vertical());
        
        let m_child = tree_nodes.get(entity).map(|v| &v.children);
        if let Some(child) = m_child {
            for centity in child {
                if let Some(elem) = elems.get(*centity) {
                   let child_size:Vector2<f64> = elem.measure(*centity, inner_size, rects, tree_nodes, elems,cells);
                   let is_static:bool = elem.fview(|v| v.view_type.is_static());
                   if child_size.x > content_size.x && is_static {
                       content_size.x = child_size.x
                   }
                   if child_size.y > content_size.y  && is_static {
                    content_size.y = child_size.y
                   }
                }
            }

            for centity in child {
                if let Some(elem) = elems.get(*centity) {
                   elem.measure(*centity, content_size, rects, tree_nodes, elems,cells);
                }
            }
        }

        
        rects.get_mut(entity).map(|rect| {
            rect.set_width(content_size.x as f32);
            rect.set_height(content_size.y as f32);
        });
        content_size
    }

    fn arrange(
        &self,
        entity: Entity,
        size: Vector2<f64>,
        rects: &mut WriteStorage<Rect2D>,
        tree_nodes: &ReadStorage<TreeNode>,
        elems: &WriteStorage<LayoutElement>,
        trans: &mut WriteStorage<Transform>,
        origin: Vector3<f32>,
        cells: &ReadStorage<GridCell>,
    ) {
        self.view.arrange(entity, size, rects, tree_nodes, elems, trans, origin, cells);
        let child_origin:Vector3<f32> = self.view.calc_orign(entity, rects);
        let (width,height) = {
            let rect = rects.get(entity).unwrap();
            (rect.width(),rect.height())
        };

        let m_child = tree_nodes.get(entity).map(|v| &v.children);
        if let Some(child) = m_child {
            for centity in child {
                if let Some(elem) = elems.get(*centity) {
                    let (child_width,child_height) = {
                        let rect = rects.get(*centity).unwrap();
                        (rect.width(),rect.height())
                    };

                    let (hor,ver,mar) = elem.fview(|v| (v.hor,v.ver,v.margin.clone()));
                    let mut new_pos:Vector2<f32> = Vector2::default();
                    match hor {
                        LayoutAlignment::Center => new_pos.x = width * 0.5f32 - (child_width * 0.5f32),
                        LayoutAlignment::End =>  new_pos.x = width - child_width - self.view.padding.right as f32 - mar.right as f32,
                        _ => new_pos.x = 0f32 + self.view.padding.left as f32
                    }

                    match ver {
                        LayoutAlignment::Center => new_pos.y = -height * 0.5f32 + child_height * 0.5f32,
                        LayoutAlignment::End => new_pos.y = -height + child_height + self.view.padding.bottom as f32 + mar.bottom as f32,
                        _ => new_pos.y = 0f32 - self.view.padding.top as f32
                    }
                    elem.fview(|v| {v.pos.set(new_pos);});
                    elem.arrange(*centity, size, rects, tree_nodes, elems, trans, child_origin,cells);
                }
            }
        }
    }
}
