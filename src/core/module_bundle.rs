use specs::{DispatcherBuilder,World};
pub trait IModuleBundle {
    fn build(world:&mut World,builder:&mut DispatcherBuilder<'static,'static>);
    fn start(&mut self,world:&mut World);
    fn update(&mut self,world:&mut World);
    fn quit(&mut self,world:&mut World);
}