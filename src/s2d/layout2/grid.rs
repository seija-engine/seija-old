use super::{View,IView};
use specs::{Entity,WriteStorage,ReadStorage};
use nalgebra::{Vector2,Vector3};
use crate::common::{Rect2D,TreeNode,Transform};
use crate::s2d::layout2::{LayoutElement};
pub struct Grid {
   pub view:View,
   pub col:Vec<CellInfo>,
   pub row:Vec<CellInfo>,
}

pub struct CellInfo {
    pub size:f32,
    pub is_rate:bool,
    pub span:u32
}

impl CellInfo {
    pub fn new(size: f32, is_rate: bool, span: u32) -> Self { Self { size, is_rate, span } }
}


impl IView for Grid {
    fn measure(&self,entity:Entity 
                    ,size:Vector2<f64>
                    ,rects:&mut WriteStorage<Rect2D>
                    ,tree_nodes:&ReadStorage<TreeNode>
                    ,elems:&WriteStorage<LayoutElement>) -> Vector2<f64> {
        
        todo!()
    }

    fn arrange(&self,entity:Entity
                    ,size:Vector2<f64>
                    ,rects:&mut WriteStorage<Rect2D>
                    ,tree_nodes:&ReadStorage<TreeNode>
                    ,elems:&WriteStorage<LayoutElement>
                    ,trans:&mut WriteStorage<Transform>, origin:Vector3<f32>) {
        todo!()
    }
}