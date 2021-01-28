pub mod transform;
pub mod component;
use specs::{World,DispatcherBuilder,WorldExt};
pub use component::{TransformSystem,HideHierarchySystem};
use crate::common::{TreeNode,Hidden,HiddenPropagate};



pub fn build_transform_module<'a,'b>(world:&mut World,builder: &mut DispatcherBuilder<'a, 'b>) {
    world.register::<Hidden>();
    world.register::<HiddenPropagate>();
    world.register::<TreeNode>();
    
   
    builder.add(TransformSystem::new(world),"transform_system",&[]);
    builder.add(HideHierarchySystem::new(world),"hide_hierarchy_system",&[]);
}