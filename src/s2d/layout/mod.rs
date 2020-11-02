mod screen_scaler;
mod system;
mod stack_panel;
mod base_layout;
pub use system::{LayoutSystem};
pub use screen_scaler::{ScaleType,ScreenScaler};
pub use base_layout::{BaseLayout};
pub use stack_panel::{StackPanel};
use specs::{World,DispatcherBuilder,WorldExt};
#[derive(Debug,Clone,Eq, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical
}

#[derive(Debug,Clone,Eq, PartialEq)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
    Stretch
}

impl From<i32> for HorizontalAlign {
    fn from(n:i32) -> Self {
        match n {
            0 => HorizontalAlign::Left,
            1 => HorizontalAlign::Center,
            2 => HorizontalAlign::Right,
            3 => HorizontalAlign::Stretch,
            _ => HorizontalAlign::Stretch
        }
    }
}

impl Default for HorizontalAlign {
    fn default() -> Self {
        HorizontalAlign::Stretch
    }
}
#[derive(Debug,Clone,Eq, PartialEq)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
    Stretch
}

impl From<i32> for VerticalAlign {
    fn from(n: i32) -> Self {
        match n {
            0 => VerticalAlign::Top,
            1 => VerticalAlign::Center,
            2 => VerticalAlign::Bottom,
            3 => VerticalAlign::Stretch,
            _ => VerticalAlign::Stretch
        }
    }
}

impl Default for VerticalAlign {
    fn default() -> Self {
        VerticalAlign::Stretch
    }
}
#[derive(Debug,Clone,Default)]
pub struct LayoutRect {
    left:f32,
    right:f32,
    top:f32,
    bottom:f32
}

impl LayoutRect {
    pub fn set_all(&mut self,num:f32) {
        self.left = num;
        self.right = num;
        self.top = num;
        self.bottom = num;
    }

    pub fn set(&mut self,l:f32,r:f32,t:f32,b:f32) {
        self.left = l;
        self.right = r;
        self.top = t;
        self.bottom = b;
    }
}

pub fn init_layout_system(world:&mut World,builder:&mut DispatcherBuilder<'static,'static>) {
    world.register::<ScreenScaler>();
    world.register::<BaseLayout>();
    world.register::<StackPanel>();
    builder.add(LayoutSystem::default(), &"layout_system", &[]);
}