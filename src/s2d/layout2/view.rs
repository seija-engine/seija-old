use super::types::{Thickness,LayoutAlignment};
use specs::{Component,DenseVecStorage};
pub trait IView {
    fn view(&self) -> &LayoutView;
}

#[derive(Clone,Default)]
pub struct LayoutView {
    pub margin:Thickness,
    pub padding:Thickness,
    pub hor:LayoutAlignment,
    pub ver:LayoutAlignment
}

impl Component for LayoutView {
    type Storage = DenseVecStorage<LayoutView>;
}


impl IView for LayoutView {
    fn view(&self) -> &LayoutView {
        self
    }
}