use super::{IView, LNumber, View};
use specs::{Entity,WriteStorage,ReadStorage,Component,DenseVecStorage};
use nalgebra::{Vector2,Vector3};
use crate::common::{Rect2D,TreeNode,Transform};
use crate::s2d::layout::{LayoutElement};
use std::cell::RefCell;

#[derive(Default)]
pub struct Grid {
   pub view:View,
   pub rows:Vec<LNumber>,
   pub cols:Vec<LNumber>,
   _row_sizes:RefCell<Vec<f32>>,
   _col_sizes:RefCell<Vec<f32>>,
}

unsafe impl Sync for Grid {}

pub struct GridCell { 
    pub col:usize, 
    pub row:usize,
    pub col_span:usize,
    pub row_span:usize,
}

impl GridCell {
    pub fn new(col: usize, row: usize, col_span: usize, row_span: usize) -> Self { Self { col, row, col_span, row_span } }
    pub fn calc_index(&self) -> (usize,usize,usize,usize) {
        let sx = self.col;
        let ex = if self.col_span > 1 { self.col + self.col_span - 1 } else { self.col };
        let sy = self.row;
        let ey = if self.row_span > 1 {  self.row + self.row_span - 1 } else {self.row };
        (sx,ex,sy,ey)
    }
}


impl Component for GridCell {
    type Storage = DenseVecStorage<GridCell>;
}

impl Grid {
    fn calc_size(&self,max:f32,lst:&Vec<LNumber>) -> Vec<f32> {
        let mut ret_list = Vec::with_capacity(lst.len());
        let mut rate_max:f32 = max;
        let mut all_rate:f32 = 0f32;
        for item in lst {
            match item {
                LNumber::Const(cn) => rate_max -= *cn,
                LNumber::Rate(rate) => all_rate += rate
            }
        }
        for item in lst {
            match item {
                LNumber::Const(cn) => {
                    ret_list.push(*cn)
                },
                LNumber::Rate(rate) => {
                    let len = rate_max * (rate / all_rate);
                    ret_list.push(len)
                }
            }
        }
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
        
        let col_sizes = self.calc_size(inner_size.x as f32, &self.cols);
        let row_sizes = self.calc_size(inner_size.y as f32, &self.rows);
        let m_childs = tree_nodes.get(entity).map(|v| &v.children);
        if row_sizes.len() > 0 && col_sizes.len() > 0 {
            if let Some(childs) = m_childs {
                for child_entity in childs {
                   if let Some(cell) = cells.get(*child_entity) {
                       let (sx,ex,sy,ey) = cell.calc_index();
                       let mut width:f32 = 0f32;
                       for idx in sx..(ex + 1) {
                           width += col_sizes[idx];
                       }
                       let mut height:f32 = 0f32;
                       for idx in sy..(ey + 1) {
                        height += row_sizes[idx];
                       }
                      elems.get(*child_entity).map(|v|
                        v.measure(*child_entity, Vector2::new(width as f64,height as f64), rects, tree_nodes, elems, cells));
                   }
                }
            }       
        }
        self._row_sizes.replace(row_sizes);
        self._col_sizes.replace(col_sizes);
        size
    }

    fn arrange(&self,entity:Entity
                    ,size:Vector2<f64>
                    ,rects:&mut WriteStorage<Rect2D>
                    ,tree_nodes:&ReadStorage<TreeNode>
                    ,elems:&WriteStorage<LayoutElement>
                    ,trans:&mut WriteStorage<Transform>
                    ,origin:Vector3<f32>
                    ,cells:&ReadStorage<GridCell>) {
        self.view.arrange(entity, size, rects, tree_nodes, elems, trans, origin,cells);
        let child_origin = self.view.calc_orign(entity, rects);
        let m_childs = tree_nodes.get(entity).map(|v| &v.children);
        if let Some(childs) = m_childs {
            for child_entity in childs {
                if let Some(cell) = cells.get(*child_entity) {
                    let (sx,_,sy,_) = cell.calc_index();
                    let mut x:f32 = 0f32;
                    for idx in 0..sx {
                        x += self._col_sizes.borrow()[idx];
                    }
                    let mut y:f32 = 0f32;
                    for idx in 0..sy {
                        y -= self._row_sizes.borrow()[idx];
                    }
                    
                    if let Some(elem) = elems.get(*child_entity) {
                        let mut new_pos:Vector3<f32> = elem.fview(|v| v.pos.get());
                        new_pos.x = x;
                        new_pos.y = y;
                        elem.fview(|v| v.pos.set(new_pos));
                        elem.arrange(*child_entity, size, rects, tree_nodes, elems, trans, child_origin,cells);
                    }
                    
                }
            }
        }
    }
}