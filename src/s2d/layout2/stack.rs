use specs::{Component, DenseVecStorage, World};
use nalgebra::Vector2;

use super::{handle::LayoutHandle, view::{LayoutView,IView}};
use super::layout::{Layout,ILayout};

impl IView for StackLayout {
    fn view(&self) -> &LayoutView { &self.layout.view() }
    fn measure(&self, size:Vector2<f32>) -> Vector2<f32> {
        todo!()
    }
}

impl ILayout for StackLayout {
    fn layout(&self) -> &Layout { &self.layout }
}

impl Component for StackLayout {
    type Storage = DenseVecStorage<StackLayout>;
}


#[derive(Default)]
pub struct StackLayout {
    layout:Layout
}

impl StackLayout {
    pub fn add_view(&mut self,world:&World,view_handle:LayoutHandle) {
        println!("StackLayout: {:?}",view_handle.entity());
        self.layout.add_view(world,view_handle)   
    }
}