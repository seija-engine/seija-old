use specs::{Component, DenseVecStorage, World};
use nalgebra::Vector2;
use super::LayoutData;
use super::{handle::LayoutHandle, view::{LayoutView}};


impl Component for StackLayout {
    type Storage = DenseVecStorage<StackLayout>;
}

#[derive(Default)]
pub struct StackLayout {
    
}

impl StackLayout {
    pub fn measure(&self,ldata:&LayoutData,size:Vector2<f64>) -> Vector2<f64> {
        size
    }
}