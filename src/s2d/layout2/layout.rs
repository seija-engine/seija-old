use super::view::{LayoutView,IView};
use super::handle::{LayoutHandle,fetch_layout_storage};
use specs::{prelude::ComponentEvent, World, WorldExt};
use nalgebra::Vector2;
use specs::{
    Component, DenseVecStorage, Entities, Entity, FlaggedStorage, ReadExpect, ReaderId, System,
    SystemData, WriteStorage,ReadStorage,Join
};
pub trait ILayout {
    fn layout(&self) -> &Layout;
}

impl IView for Layout {
    fn view(&self) -> &LayoutView { &self.view }
    fn measure(&self, size:Vector2<f32>) -> Vector2<f32> {
        todo!()
    }
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
    pub fn add_view(&mut self,world:&World,handle:LayoutHandle) {
        let mut s = world.write_storage::<LayoutView>();
        let reader = s.register_reader();
        
        self._children.push(handle)
    }
}