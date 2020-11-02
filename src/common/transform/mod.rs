pub mod transform;
pub mod component;
use specs::{World,DispatcherBuilder,WorldExt};
use specs_hierarchy::{HierarchySystem};
pub use component::{Parent,TransformSystem,ParentHierarchy,HideHierarchySystem};
use crate::common::{Hidden,HiddenPropagate};



pub fn build_transform_module<'a,'b>(world:&mut World,builder: &mut DispatcherBuilder<'a, 'b>) {
    world.register::<Hidden>();
    world.register::<HiddenPropagate>();
    builder.add(HierarchySystem::<Parent>::new(world),"parent_hierarchy_system",&[]);
    builder.add(TransformSystem::new(world),"transform_system",&["parent_hierarchy_system"]);
    builder.add(HideHierarchySystem::new(world),"hide_hierarchy_system",&[]);
}