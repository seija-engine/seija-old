use super::view::{LayoutView,IView};
use super::handle::{LayoutHandle,fetch_layout_storage};
use specs::{World,Component,DenseVecStorage};
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

impl Component for Layout {
    type Storage = DenseVecStorage<Layout>;
}

impl Layout {
    pub fn add_view(&mut self,handle:LayoutHandle,world:&World) {
        let ls = fetch_layout_storage(world);
        let view = handle.view(&ls).unwrap();
        self._children.push(handle)
    }
}