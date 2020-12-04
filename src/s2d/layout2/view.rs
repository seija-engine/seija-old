use super::types::{Thickness,LayoutAlignment};
pub trait IView {
    fn view(&self) -> &LayoutView;
}

#[derive(Clone,Default)]
pub struct LayoutView {
    margin:Thickness,
    padding:Thickness,
    hor:LayoutAlignment,
    ver:LayoutAlignment
}


impl IView for LayoutView {
    fn view(&self) -> &LayoutView {
        self
    }
}