use crate::{common::{Rect2D, Transform, Tree, TreeEvent, TreeNode}, window::ViewPortSize};
use hibitset::BitSet;
use specs::{System,World,ReadExpect,Entity,ReadStorage,WriteStorage,Entities};
use shrev::{ReaderId};
use nalgebra::{Vector2,Vector3};

use super::{GridCell, IView, LayoutElement, Stack};
/*
    root0 (LayoutView,StackPanel)
      img0 (LayoutView)
      img1 (LayoutView)
      panel (LayoutView,StackPanel)
        imga (LayoutView)
        imgb (LayoutView)
        imgc (LayoutView)
    
    grid (LayoutView,Grid)
      grid0 (LayoutView,GridCell)
        panel (LayoutView,StackPanel)
          imga (LayoutView)
          imgb (LayoutView)
      grid1 (LayoutView,GridCell)
        panel (LayoutView,StackPanel)
          imga (LayoutView)
          imgb (LayoutView)
*/

pub struct LayoutSystem {
   ev_tree:ReaderId<TreeEvent>,
   modified:BitSet,
}

impl LayoutSystem {
    pub fn new(world:&mut World) -> LayoutSystem {
        let tree = world.get_mut::<Tree>().unwrap();
        LayoutSystem {
           ev_tree:tree.channel.register_reader(),
           modified:BitSet::new()
        }
    }

    pub fn on_dirty(&mut self,tree_nodes:&ReadStorage<TreeNode>,_parent:Option<Entity>,entity:Entity) {
        let mut cur_entity = entity;
        while self.is_invalid_measure(cur_entity) {
           if let Some(node) = tree_nodes.get(cur_entity) {
               if let Some(parent) = node.parent {
                   cur_entity = parent;
               } else {
                   self.modified.add(cur_entity.id());
                   break;
               }
           } else {
               self.modified.add(cur_entity.id());
               break;
           }
        }
    }

  
    fn is_invalid_measure(&self,_entity:Entity) -> bool {
        true
    }
}

pub type LayoutData<'a> = (
    Entities<'a>,
    ReadExpect<'a,Tree>,
    ReadStorage<'a,TreeNode>,
    WriteStorage<'a,LayoutElement>,
    ReadExpect<'a,ViewPortSize>,
    WriteStorage<'a,Rect2D>,
    WriteStorage<'a,Transform>,
    ReadStorage<'a,GridCell>);


impl<'a> System<'a> for LayoutSystem {
    type SystemData = LayoutData<'a>;
    
    fn run(&mut self,mut ldata: Self::SystemData) {
       self.modified.clear();
      
    
       for ev in ldata.1.channel.read(&mut self.ev_tree) {
           let tree_nodes = &ldata.2;
           match ev {
            TreeEvent::Add(p,e) => self.on_dirty(&tree_nodes,*p, *e),
            TreeEvent::Remove(p,_) => {
                if let Some(pentity) = p {
                    let pparent = tree_nodes.get(*pentity).and_then(|n| n.parent);
                    self.on_dirty(&tree_nodes,pparent, *pentity);
                }
            }
           }
       }
       let iter = self.modified.clone().into_iter();
       for eid in iter {
           let cur_entity = ldata.0.entity(eid);
           let elem = ldata.3.get(cur_entity).unwrap();
          
           elem.update_layout(cur_entity,&ldata.2
                              ,&mut ldata.5
                              ,&ldata.3
                              ,&ldata.4
                              ,&mut ldata.6
                              ,&ldata.7);  
       }
    }
}