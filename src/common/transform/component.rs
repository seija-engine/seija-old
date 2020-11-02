use crate::common::{Transform,HiddenPropagate};
use hibitset::BitSet;
use specs::storage::ComponentEvent;
use specs::{
    Component, DenseVecStorage, Entities, Entity, FlaggedStorage, ReadExpect, ReaderId, System,
    SystemData, World, WriteStorage,ReadStorage,Join
};
use specs_hierarchy::HierarchyEvent;
use specs_hierarchy::{Hierarchy, Parent as HParent};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

pub type ParentHierarchy = Hierarchy<Parent>;

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Parent {
    pub entity: Entity,
}

impl Component for Parent {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

impl HParent for Parent {
    fn parent_entity(&self) -> Entity {
        self.entity
    }
}

impl Component for Transform {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

pub struct TransformSystem {
    local_modified: BitSet,
    locals_events_id: ReaderId<ComponentEvent>,
    parent_events_id: ReaderId<HierarchyEvent>,
}

impl TransformSystem {
    pub fn new(world: &mut World) -> Self {
        <TransformSystem as System<'_>>::SystemData::setup(world);
        let mut hierarchy = world.fetch_mut::<ParentHierarchy>();
        let mut locals = WriteStorage::<Transform>::fetch(&world);
        let parent_events_id = hierarchy.track();
        let locals_events_id = locals.register_reader();
        TransformSystem {
            local_modified: BitSet::new(),
            locals_events_id,
            parent_events_id,
        }
    }
}

impl<'a> System<'a> for TransformSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, ParentHierarchy>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, Parent>,
    );
    fn run(&mut self,(entities, hierarchy, mut locals, parents): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("TransformSystem run");
        self.local_modified.clear();
        locals.channel().read(&mut self.locals_events_id)
                        .for_each(|event| match event {
                            ComponentEvent::Inserted(id) | ComponentEvent::Modified(id) => {
                                self.local_modified.add(*id);
                            }
                            ComponentEvent::Removed(_id) => {}
                        });
        for event in hierarchy.changed().read(&mut self.parent_events_id) {
            match *event {
                HierarchyEvent::Removed(entity) => {
                    let _ = entities.delete(entity);
                },
                HierarchyEvent::Modified(entity) => {
                    self.local_modified.add(entity.id());
                }
            }
        }
        let mut modified = Vec::new();
        for (entity, _, local, _) in (&*entities, &self.local_modified, &mut locals, !&parents).join() {
            modified.push(entity.id());
            local.global_matrix = local.matrix();
        }
        modified.into_iter().for_each(|id| { self.local_modified.add(id); });
        for entity in hierarchy.all() {
            let self_dirty = self.local_modified.contains(entity.id());
            if let Some(parent) = parents.get(*entity) {
                let parent_dirty = self.local_modified.contains(parent.entity.id());
                if parent_dirty || self_dirty {
                    let combined_transform = {
                        let local = locals.get(*entity);
                        if local.is_none() {
                            continue;
                        }
                        let local = local.unwrap();
                        if let Some(parent_global) = locals.get(parent.entity) {
                            parent_global.global_matrix * local.matrix()
                        } else {
                            local.matrix()
                        }
                    };
                    self.local_modified.add(entity.id());
                    locals.get_mut(*entity).expect("unreachable: We know this entity has a local because is was just modified.").global_matrix = combined_transform;
                }
            }
        }
        locals.channel().read(&mut self.locals_events_id);
    }
}


#[derive(Debug)]
pub struct HideHierarchySystem {
    marked_as_modified: BitSet,
    hidden_events_id: ReaderId<ComponentEvent>,
    parent_events_id: ReaderId<HierarchyEvent>,
}

impl HideHierarchySystem {
    pub fn new(world:&mut World) -> Self {
        let mut hidden = WriteStorage::<HiddenPropagate>::fetch(&world);
        let hidden_events_id = hidden.register_reader();
        HideHierarchySystem {
            marked_as_modified: BitSet::default(),
            hidden_events_id: hidden_events_id,
            parent_events_id: world.fetch_mut::<ParentHierarchy>().track()
        }
    }
}


impl<'a> System<'a> for HideHierarchySystem {
    type SystemData = (
        WriteStorage<'a, HiddenPropagate>,
        ReadStorage<'a, Parent>,
        ReadExpect<'a, ParentHierarchy>,
    );

    fn run(&mut self, (mut hidden, parents, hierarchy): Self::SystemData) {
        self.marked_as_modified.clear();
        let self_hidden_events_id = &mut self.hidden_events_id;
        let self_marked_as_modified = &mut self.marked_as_modified;

        hidden.channel().read(self_hidden_events_id).for_each(|event| match event {
            ComponentEvent::Inserted(id) | ComponentEvent::Removed(id) => {
                self_marked_as_modified.add(*id);
            }
            ComponentEvent::Modified(_id) => {}
        });

        for event in hierarchy.changed().read(&mut self.parent_events_id) {
            match *event {
                HierarchyEvent::Removed(entity) => {
                    self_marked_as_modified.add(entity.id());
                }
                HierarchyEvent::Modified(entity) => {
                    self_marked_as_modified.add(entity.id());
                }
            }
        }

        for entity in hierarchy.all() {
            {
                let self_dirty = self_marked_as_modified.contains(entity.id());
                let parent_entity = parents.get(*entity).expect("Unreachable: All entities in `ParentHierarchy` should also be in `Parents`").entity;
                let parent_dirty = self_marked_as_modified.contains(parent_entity.id());
                if parent_dirty {
                    if hidden.contains(parent_entity) {
                        for child in hierarchy.all_children_iter(parent_entity) {
                            if let Err(e) = hidden.insert(child, HiddenPropagate::default()) {
                                eprintln!("Failed to automatically add `HiddenPropagate`: {:?}", e);
                            };
                        }
                    } else {
                        for child in hierarchy.all_children_iter(parent_entity) {
                            hidden.remove(child);
                        }
                    }
                } else if self_dirty {
                    if hidden.contains(*entity) {
                        for child in hierarchy.all_children_iter(*entity) {
                            if let Err(e) = hidden.insert(child, HiddenPropagate::default()) {
                                eprintln!("Failed to automatically add `HiddenPropagate`: {:?}", e);
                            };
                        }
                    } else {
                        for child in hierarchy.all_children_iter(*entity) {
                            hidden.remove(child);
                        }
                    }
                }
            };

            hidden.channel().read(self_hidden_events_id).for_each(|event| match event {
                ComponentEvent::Inserted(id) | ComponentEvent::Removed(id) => {
                    self_marked_as_modified.add(*id);
                }
                ComponentEvent::Modified(_id) => {}
            });
        }
    }

}