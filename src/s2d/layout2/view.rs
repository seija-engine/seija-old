use super::types::{Thickness,LayoutAlignment};
use specs::{Component,FlaggedStorage};
use nalgebra::Vector2;
use super::LayoutData;

#[derive(Clone,Default)]
pub struct LayoutView {
    pub size:Vector2<f64>,
    pub margin:Thickness,
    pub padding:Thickness,
    pub hor:LayoutAlignment,
    pub ver:LayoutAlignment
}

impl Component for LayoutView {
    type Storage = FlaggedStorage<LayoutView>;
}

impl LayoutView {
    pub fn measure(&self,ldata:&LayoutData,size:Vector2<f64>) -> Vector2<f64> {
        size
    }
}