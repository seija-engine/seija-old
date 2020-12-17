use hibitset::BitSet;
use shrev::EventChannel;
use specs::{Component, DenseVecStorage, Entity, ReadStorage, World, WorldExt, WriteStorage};
use std::collections::VecDeque;
#[derive(Default)]
pub struct TreeNode {
    pub parent: Option<Entity>,
    pub children: Vec<Entity>,
}

impl TreeNode {
    pub fn new(parent: Option<Entity>) -> TreeNode {
        TreeNode {
            parent,
            children: vec![],
        }
    }

    pub fn remove(&mut self, e: &Entity) {
        let index = self.children.iter().position(|c| c.id() == e.id());
        if let Some(idx) = index {
            self.children.remove(idx);
        }
    }
}

impl Component for TreeNode {
    type Storage = DenseVecStorage<TreeNode>;
}

pub enum TreeEvent {
    Add(Option<Entity>, Entity),
    Remove(Option<Entity>, Entity),
    Update(Option<Entity>, Option<Entity>, Entity),
}

#[derive(Default)]
pub struct Tree {
    roots: Vec<Entity>,
    pub channel: EventChannel<TreeEvent>,
}

impl Tree {
    pub fn roots(&self) -> &Vec<Entity> {
        &self.roots
    }

    pub fn parent(world: &mut World, entity: Entity) -> Option<Entity> {
        let mut s_tree_node = world.write_storage::<TreeNode>();
        let node = s_tree_node.get_mut(entity).unwrap();
        node.parent
    }

    pub fn update(world: &mut World, entity: Entity, parent: Option<Entity>) -> Entity {
        let mut oldp: Option<Entity> = None;
        {
            let mut tree_nodes: WriteStorage<TreeNode> = world.write_storage::<TreeNode>();
            if !tree_nodes.contains(entity)
                || parent.map(|e| !tree_nodes.contains(e)).unwrap_or(false)
            {
                return entity;
            }
            if let Some(old_parent) = tree_nodes.get(entity).unwrap().parent {
                oldp = Some(old_parent);
                tree_nodes.get_mut(old_parent).unwrap().remove(&entity);
            } else {
                let index = world
                    .fetch_mut::<Tree>()
                    .roots
                    .iter()
                    .position(|c| c.id() == entity.id());
                if let Some(index) = index {
                    world.fetch_mut::<Tree>().roots.remove(index);
                }
            }
            if let Some(p) = parent {
                tree_nodes.get_mut(p).unwrap().children.push(entity);
            } else {
                drop(tree_nodes);
                let tree = world.get_mut::<Tree>().unwrap();
                tree.roots.push(entity);
            }
        };

        let tree = world.get_mut::<Tree>().unwrap();
        tree.channel
            .single_write(TreeEvent::Update(oldp, parent, entity));
        entity
    }

    pub fn add(world: &mut World, entity: Entity, parent: Option<Entity>) -> Entity {
        let new_node = TreeNode::new(parent);
        let mut s_tree_node: WriteStorage<TreeNode> = world.write_storage::<TreeNode>();
        s_tree_node.insert(entity, new_node).unwrap();
        if let Some(p) = parent {
            if let Some(pt) = s_tree_node.get_mut(p) {
                pt.children.push(entity);
            }
            drop(s_tree_node);
            world
                .get_mut::<Tree>()
                .unwrap()
                .channel
                .single_write(TreeEvent::Add(Some(p), entity));
        } else {
            drop(s_tree_node);
            let tree = world.get_mut::<Tree>().unwrap();
            tree.roots.push(entity);
            tree.channel.single_write(TreeEvent::Add(None, entity));
        }
        entity
    }

    pub fn remove_from_parent(world: &mut World,entity: Entity,is_destory: bool) -> Option<Entity> {
        let parent = {
            let mut tree_nodes: WriteStorage<TreeNode> = world.write_storage::<TreeNode>();
            if !tree_nodes.contains(entity) {
                return Some(entity);
            };
            let parent = tree_nodes.get(entity).unwrap().parent;
            if let Some(parent) = parent {
                tree_nodes.get_mut(parent).unwrap().remove(&entity);
            } else {
                let index = world.fetch_mut::<Tree>().roots.iter().position(|c| c.id() == entity.id());
                if let Some(index) = index {
                    world.fetch_mut::<Tree>().roots.remove(index);
                }
            }
            parent
        };
        let tree = world.get_mut::<Tree>().unwrap();
        tree.channel.single_write(TreeEvent::Remove(parent, entity));
        if is_destory {
            Tree::destory(world, entity);
            None
        } else {
            Some(entity)
        }
    }


    fn destory(world: &mut World, entity: Entity) {
        let mut rm_list: Vec<Entity> = vec![];
        {
            let mut q_list: VecDeque<Entity> = VecDeque::new();
            q_list.push_back(entity);
            let s_node_tree = world.write_storage::<TreeNode>();

            while let Some(ce) = q_list.pop_front() {
                rm_list.push(ce);
                s_node_tree.get(ce).unwrap().children.iter().for_each(|e| {
                    q_list.push_back(*e);
                });
            }
        }
        world.delete_entities(&rm_list).unwrap();
    }

    pub fn all_children(tree_nodes: &ReadStorage<TreeNode>, entity: Entity) -> BitSet {
        let mut set = BitSet::new();
        Tree::add_children_to_set(entity, &mut set, &tree_nodes);
        set
    }

    fn add_children_to_set(entity: Entity, set: &mut BitSet, tree_nodes: &ReadStorage<TreeNode>) {
        let mnode = tree_nodes.get(entity);
        if let Some(node) = mnode {
            for c in node.children.iter() {
                set.add(c.id());
                Tree::add_children_to_set(*c, set, tree_nodes);
            }
        }
    }

    pub fn all_sort_children(tree_nodes: &ReadStorage<TreeNode>, entity: Entity) -> BitSet {
        let mut set = BitSet::new();
        let mut q_list: VecDeque<Entity> = VecDeque::new();
        if let Some(cnode) = tree_nodes.get(entity) {
            cnode.children.iter().for_each(|e| {
                q_list.push_back(*e);
            });
        }
        while let Some(ce) = q_list.pop_front() {
            set.add(ce.id());
            tree_nodes.get(ce).unwrap().children.iter().for_each(|e| {
                q_list.push_back(*e);
            });
        }
        set
    }
}
