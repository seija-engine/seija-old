use super::view::{LayoutView,IView};
use super::handle::LayoutHandle;
pub trait ILayout {
    fn layout(&self) -> &Layout;
}

impl IView for Layout {
    fn view(&self) -> &LayoutView { &self.view }
}

impl ILayout for Layout {
    fn layout(&self) -> &Layout { self }
}

#[derive(Default)]
pub struct Layout {
    view:LayoutView,
    _children:Vec<LayoutHandle>
}

impl Layout {
    pub fn add_view(&mut self,handle:LayoutHandle) {
        self._children.push(handle)
    }
}