use super::{IView, LNumber, View};
use specs::{Entity,WriteStorage,ReadStorage,Component,DenseVecStorage};
use nalgebra::{Vector2,Vector3};
use crate::common::{Rect2D,TreeNode,Transform};
use crate::s2d::layout2::{LayoutElement};
pub struct Grid {
   pub view:View,
   pub rows:Vec<LNumber>,
   pub cols:Vec<LNumber>,
   
}

pub struct GridCell { 
    pub col:u16, 
    pub row:u16,
    pub col_span:u16,
    pub row_span:u16
}

impl GridCell {
    pub fn new(col: u16, row: u16, col_span: u16, row_span: u16) -> Self { Self { col, row, col_span, row_span } }
}


impl Component for GridCell {
    type Storage = DenseVecStorage<GridCell>;
}

impl Grid {
    fn calc_size(&self,max:f32,lst:&Vec<LNumber>) -> Vec<f32> {
        let ret_list = Vec::with_capacity(lst.len());
        
        ret_list
    }
}

impl IView for Grid {
    fn measure(&self,entity:Entity 
                    ,size:Vector2<f64>
                    ,rects:&mut WriteStorage<Rect2D>
                    ,tree_nodes:&ReadStorage<TreeNode>
                    ,elems:&WriteStorage<LayoutElement>
                    ,cells:&ReadStorage<GridCell>) -> Vector2<f64> {
        let content_size:Vector2<f64> = self.view.measure(entity, size, rects, tree_nodes, elems,cells);
        let inner_size:Vector2<f64> = Vector2::new(content_size.x - self.view.padding.horizontal(),
                                                   content_size.y - self.view.padding.vertical());
        
        let m_childs = tree_nodes.get(entity).map(|v| &v.children);
        if let Some(childs) = m_childs {
            for child_entity in childs {
               if let Some(cell) = cells.get(*child_entity) {
               }
            }
        }
        size
    }

    fn arrange(&self,entity:Entity
                    ,size:Vector2<f64>
                    ,rects:&mut WriteStorage<Rect2D>
                    ,tree_nodes:&ReadStorage<TreeNode>
                    ,elems:&WriteStorage<LayoutElement>
                    ,trans:&mut WriteStorage<Transform>, origin:Vector3<f32>) {
       
    }
}