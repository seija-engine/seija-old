use std::collections::VecDeque;

use hibitset::BitSet;
use specs::{Entity, WorldExt,World,ReadStorage,DenseVecStorage,Component};
use shrev::{EventChannel};
#[derive(Default)]
pub struct TreeNode {
  pub parent:Option<Entity>,
  pub children:Vec<Entity>
}

impl TreeNode {
    pub fn new(parent:Option<Entity>) -> TreeNode {
        TreeNode {parent ,children:vec![] }
    }

    pub fn remove(&mut self,e:&Entity) {
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
    Add(Option<Entity>,Entity),
    Remove(Option<Entity>,Entity)
}

#[derive(Default)]
pub struct Tree {
    roots:Vec<Entity>,
    pub channel:EventChannel<TreeEvent>
}

impl Tree {

    pub fn roots(&self) -> &Vec<Entity> {
        &self.roots
    }

    pub fn parent(world:&mut World,entity:Entity) -> Option<Entity> {
        let mut s_tree_node = world.write_storage::<TreeNode>();
        let node = s_tree_node.get_mut(entity).unwrap();
        node.parent
    }

    pub fn add(world:&mut World,entity:Entity,parent:Option<Entity>) -> Entity {
        let new_node = TreeNode::new(parent);
        let mut s_tree_node = world.write_storage::<TreeNode>();
        s_tree_node.insert(entity, new_node).unwrap();
        if let Some(p) = parent {
            s_tree_node.get_mut(p).unwrap().children.push(entity);
            drop(s_tree_node);
            world.get_mut::<Tree>().unwrap().channel.single_write(TreeEvent::Add(Some(p),entity));
        } else {
            drop(s_tree_node);
            let tree = world.get_mut::<Tree>().unwrap();
            tree.roots.push(entity);
            tree.channel.single_write(TreeEvent::Remove(None,entity));
        }
        entity
    }

    pub fn remove(world:&mut World,entity:Entity,is_destory:bool) -> Option<Entity> {
        {
            let mut s_node_tree = world.write_storage::<TreeNode>();
            let cur_node = s_node_tree.get(entity).unwrap();
            if let Some(p) = cur_node.parent {
                s_node_tree.get_mut(p).unwrap().remove(&entity);
            }
        }
        
        if is_destory {
            Tree::destory(world, entity);
            None
        } else {
            Some(entity)
        }
    }

    fn destory(world:&mut World,entity:Entity) {
        let mut rm_list:Vec<Entity> = vec![];
        {
            let mut q_list:VecDeque<Entity> = VecDeque::new();
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

    pub fn all_children(tree_nodes:&ReadStorage<TreeNode>,entity:Entity) -> BitSet {
        let mut set = BitSet::new();
        Tree::add_children_to_set(entity, &mut set,&tree_nodes);
        set
    }

    fn add_children_to_set(entity: Entity, set: &mut BitSet,tree_nodes: &ReadStorage<TreeNode>) {
        let mnode = tree_nodes.get(entity);
        if let Some(node) = mnode {
            for c in node.children.iter() {
                set.add(c.id());
                Tree::add_children_to_set(*c,set,tree_nodes);
            }
        }
    }
    
}