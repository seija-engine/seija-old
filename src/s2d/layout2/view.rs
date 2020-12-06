use super::types::{Thickness,LayoutAlignment};
use specs::{Component,FlaggedStorage};
use nalgebra::Vector2;
pub trait IView {
    fn view(&self) -> &LayoutView;
    fn measure(&self,size:Vector2<f32>) -> Vector2<f32>;
}

#[derive(Clone,Default)]
pub struct LayoutView {
    pub margin:Thickness,
    pub padding:Thickness,
    pub hor:LayoutAlignment,
    pub ver:LayoutAlignment
}

impl Component for LayoutView {
    type Storage = FlaggedStorage<LayoutView>;
}


impl IView for LayoutView {
    fn view(&self) -> &LayoutView {
        self
    }
    
    fn measure(&self, size:Vector2<f32>) -> Vector2<f32> {
        todo!()
    }
}