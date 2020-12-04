use super::view::{LayoutView,IView};
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
    _children:Vec<Box<dyn IView>>
}

impl Layout {
    pub fn add_view(&mut self,view:Box<dyn IView>) {
        self._children.push(view)
    }
}