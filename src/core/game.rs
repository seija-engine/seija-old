use specs::World;
pub trait IGame {
    fn start(&mut self,world:&mut World);
    fn update(&mut self,world:&mut World);
    fn quit(&mut self,world:&mut World);
}