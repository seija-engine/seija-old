use super::view::{LayoutView,IView};
use super::layout::{Layout,ILayout};

impl IView for StackLayout {
    fn view(&self) -> &LayoutView { &self.layout.view() }
}

impl ILayout for StackLayout {
    fn layout(&self) -> &Layout { &self.layout }
}

#[derive(Default)]
pub struct StackLayout {
    layout:Layout
}