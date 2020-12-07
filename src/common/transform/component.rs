use crate::common::{HiddenPropagate, Transform, Tree, TreeEvent, TreeNode};
use hibitset::BitSet;
use specs::storage::ComponentEvent;
use specs::{
    Component, DenseVecStorage, Entities, Entity, FlaggedStorage, ReadExpect, ReaderId, System,
    SystemData, World, WriteStorage,ReadStorage,Join
};
use specs_hierarchy::HierarchyEvent;
use specs_hierarchy::{Hierarchy, Parent as HParent};
use hibitset::BitSetLike;

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



pub struct HideHierarchySystem {
    marked_as_modified: BitSet,
    only_add:BitSet,
    hidden_events_id: ReaderId<ComponentEvent>,
    tree_events_id: ReaderId<TreeEvent>,
}

impl HideHierarchySystem {
    pub fn new(world:&mut World) -> Self {
        let mut hidden = WriteStorage::<HiddenPropagate>::fetch(&world);
        let hidden_events_id = hidden.register_reader();
        HideHierarchySystem {
            only_add:BitSet::new(),
            marked_as_modified: BitSet::default(),
            hidden_events_id: hidden_events_id,
            tree_events_id: world.fetch_mut::<Tree>().channel.register_reader()        }
    }
}


impl<'a> System<'a> for HideHierarchySystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HiddenPropagate>,
        ReadStorage<'a, TreeNode>,
        ReadExpect<'a, Tree>,
    );

    fn run(&mut self, (entities,mut hiddens, tree_nodes, tree): Self::SystemData) {
        self.marked_as_modified.clear();
        self.only_add.clear();
        let self_hidden_events_id = &mut self.hidden_events_id;
        

        hiddens.channel().read(self_hidden_events_id).for_each(|event| match event {
            ComponentEvent::Inserted(id) | ComponentEvent::Removed(id) => {
                self.marked_as_modified.add(*id);
            }
            ComponentEvent::Modified(_id) => ()
        });

        for event in tree.channel.read(&mut self.tree_events_id) {
            match *event {
                TreeEvent::Add(_,entity) => {
                    self.marked_as_modified.add(entity.id());
                    self.only_add.add(entity.id());
                },
                _ => {}
            }
        }
        
        let bit_iter = self.marked_as_modified.clone().into_iter();
        for me in bit_iter {
           let cur_entity = entities.entity(me);
           let is_hide = hiddens.contains(cur_entity);
           let parent_enitity = tree_nodes.get(cur_entity).and_then(|t| t.parent);
           let is_parent_hide = parent_enitity.map(|p| hiddens.contains(p)).unwrap_or(false);
           if is_parent_hide {
              for child in Tree::all_children(&tree_nodes, parent_enitity.unwrap()).iter() {
                  let centity = entities.entity(child);
                  if !hiddens.contains(centity) {
                     if let Err(err) = hiddens.insert(centity, HiddenPropagate::default()) {
                        eprintln!("Failed to automatically add `HiddenPropagate`: {:?}", err);
                     }
                  }
              }
           } else {

            if self.only_add.contains(cur_entity.id()) && !is_hide {
               continue;
            }

            for child in Tree::all_children(&tree_nodes, cur_entity).iter() {
                let centity = entities.entity(child);
                if is_hide {
                    if !hiddens.contains(centity) {
                        if let Err(err) = hiddens.insert(centity, HiddenPropagate::default()) {
                            eprintln!("Failed to automatically add `HiddenPropagate`: {:?}", err);
                         }
                    }
                } else {
                    if hiddens.contains(centity) {
                        hiddens.remove(centity);
                    }
                }
            }
           }
        }
        
       
        /*

        for entity in hierarchy.all() {
            {
                let self_dirty = self_marked_as_modified.contains(entity.id());
                let parent_entity = parents.get(*entity).expect("Unreachable: All entities in `ParentHierarchy` should also be in `Parents`").entity;
                let parent_dirty = self_marked_as_modified.contains(parent_entity.id());
                if parent_dirty {
                    if hidden.contains(parent_entity) {
                        for child in tree.all_children_iter(parent_entity) {
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
        }*/
    }

}