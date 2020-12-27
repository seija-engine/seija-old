use crate::common::{HiddenPropagate, Transform, Tree, TreeEvent, TreeNode};
use hibitset::BitSet;
use nalgebra::{Matrix, Matrix4};
use specs::storage::ComponentEvent;
use specs::{
    Component, DenseVecStorage, Entities, Entity, FlaggedStorage, ReadExpect, ReaderId, System,
    SystemData, World, WriteStorage,ReadStorage
};

use hibitset::BitSetLike;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;


impl Component for Transform {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

pub struct TransformSystem {
    local_modified: BitSet,
    locals_events_id: ReaderId<ComponentEvent>,
    tree_events_id: ReaderId<TreeEvent>,
}

impl TransformSystem {
    pub fn new(world: &mut World) -> Self {
        <TransformSystem as System<'_>>::SystemData::setup(world);
        let mut tree = world.fetch_mut::<Tree>();
        let mut locals = WriteStorage::<Transform>::fetch(&world);
        let tree_events_id = tree.channel.register_reader();
        let locals_events_id = locals.register_reader();
        TransformSystem {
            local_modified: BitSet::new(),
            locals_events_id,
            tree_events_id
        }
    }

    pub fn is_parent_change<'a>(&self,entity:Entity,tree_nodes:&ReadStorage<'a, TreeNode>) -> bool {
        let mut node = tree_nodes.get(entity).and_then(|t| t.parent);
        while let Some(n) = node {
            if self.local_modified.contains(n.id()) {
                return true;
            }
            node = tree_nodes.get(n).and_then(|t| t.parent);
        }
        false
    }
}

impl<'a> System<'a> for TransformSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, Tree>,
        WriteStorage<'a, Transform>,
        ReadStorage<'a, TreeNode>,
    );

    fn run(&mut self,(entities, tree, mut locals, tree_nodes): Self::SystemData) {
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
        for event in tree.channel.read(&mut self.tree_events_id) {
            match *event {
                TreeEvent::Add(_,entity) => {
                    if locals.contains(entity) {
                        self.local_modified.add(entity.id());
                    }
                },
                TreeEvent::Remove(_,entity) => {
                    self.local_modified.remove(entity.id());
                },
                TreeEvent::Update(_,_,entity) => {
                    if locals.contains(entity) {
                        self.local_modified.add(entity.id());
                    }
                }
            }
        }
        
        let change_iter = self.local_modified.clone().iter();
        for e in change_iter {
            let entity = entities.entity(e);
            if self.is_parent_change(entity,&tree_nodes) {
                continue;
            }
            if let Some(p) = tree_nodes.get(entity).and_then(|v| v.parent) {
                let new_mat:Matrix4<f32> = (locals.get(p).unwrap().global_matrix) * (locals.get(entity).unwrap().matrix());
                locals.get_mut(entity).unwrap().global_matrix = new_mat;
            } else {
                let t = locals.get_mut(entity).unwrap();
                t.global_matrix = t.matrix();
            }
           
            for child in Tree::all_sort_children(&tree_nodes, entity) {
                let centity = entities.entity(child);
                let parent_entity = tree_nodes.get(centity).unwrap().parent.unwrap();
                if let Some(trans) = locals.get(parent_entity) {
                    if let Some(ctrans) = locals.get(centity) {
                        let new_mat:Matrix4<f32> = trans.global_matrix * ctrans.matrix();
                        locals.get_mut(centity).unwrap().global_matrix = new_mat;
                    }
                    
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
        

        hiddens.channel().read(&mut self.hidden_events_id).for_each(|event| match event {
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

        hiddens.channel().read(&mut self.hidden_events_id);
    }

}