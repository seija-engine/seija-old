use crate::{common::{Rect2D, Transform, Tree, TreeEvent, TreeNode}, render::components::Mesh2D, window::ViewPortSize};
use hibitset::BitSet;
use specs::{Entities, Entity, ReadExpect, ReadStorage, System, SystemData, World, WriteStorage, prelude::ComponentEvent};
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
   ev_view:ReaderId<ComponentEvent>,
   modified:BitSet,
}

impl LayoutSystem {
    pub fn new(world:&mut World) -> LayoutSystem {
        let tree = world.get_mut::<Tree>().unwrap().channel.register_reader();
        let ev_view = WriteStorage::<LayoutElement>::fetch(world).channel_mut().register_reader();
        LayoutSystem {
           ev_tree:tree,
           ev_view,
           modified:BitSet::new()
        }
    }

    pub fn on_dirty(&mut self,tree_nodes:&ReadStorage<TreeNode>,entity:Entity) {
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
       for ev in ldata.3.channel().read(&mut self.ev_view) {
           match ev {
               ComponentEvent::Modified(e) => {
                 let entity = ldata.0.entity(*e);
                 self.on_dirty(&ldata.2, entity);
               }
               _ => ()
           }
       }
    
       for ev in ldata.1.channel.read(&mut self.ev_tree) {
           let tree_nodes = &ldata.2;
           match ev {
            TreeEvent::Add(_,e) => self.on_dirty(&tree_nodes, *e),
            TreeEvent::Remove(p,_) => {
                if let Some(pentity) = p {
                    self.on_dirty(&tree_nodes, *pentity);
                }
            },
            TreeEvent::Update(oldp,p,_e) => {
                if let Some(oldp) = oldp {
                    self.on_dirty(&tree_nodes, *oldp);
                }
                if let Some(p) = p {
                    self.on_dirty(&tree_nodes, *p);
                }
            },
           }
       }
      

       let iter = self.modified.clone().into_iter();
       for eid in iter {
           let cur_entity = ldata.0.entity(eid);
           if  !ldata.5.contains(cur_entity) || !ldata.6.contains(cur_entity) {
               continue;
           }

         
           if let Some(elem) =  ldata.3.get(cur_entity) {
            elem.update_layout(cur_entity,&ldata.2
                ,&mut ldata.5
                ,&ldata.3
                ,&ldata.4
                ,&mut ldata.6
                ,&ldata.7);
           }
       }

       ldata.3.channel().read(&mut self.ev_view);
    }
}