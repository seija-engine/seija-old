use shred::{DispatcherBuilder, World};
use specs::{WorldExt};

pub mod types;
pub mod view;
pub mod layout;
pub mod stack;
pub mod handle;
pub mod system;

pub use stack::{StackLayout};
pub use view::{LayoutView};
pub use handle::{LayoutHandle};
pub use types::{LayoutAlignment,Thickness};

pub fn init_layout_system(world:&mut World,builder:&mut DispatcherBuilder<'static,'static>) {
    world.register::<StackLayout>();
    world.register::<LayoutView>();
    let layout_system = system::LayoutSystem::new(world);
    builder.add(layout_system, "layout", &[]);
}