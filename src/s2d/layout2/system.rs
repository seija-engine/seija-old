use crate::{common::{Rect2D, Tree, TreeEvent, TreeNode}, window::ViewPortSize};
use hibitset::BitSet;
use specs::{System,World,ReadExpect,Entity,ReadStorage,WriteStorage,Entities};
use shrev::{ReaderId};
use nalgebra::Vector2;

use super::{LayoutView, StackLayout};

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

    fn size_request(&self,ldata:&LayoutData,entity:Entity) -> Vector2<f64> {
        let tree_nodes = &ldata.2;
        let size:Vector2<f64> = ldata.3.get(entity).map(|view|view.size).unwrap_or(Vector2::zeros());
        if size.magnitude() > 0.01f64 {
            return size;
        }
        if let Some(parent) = tree_nodes.get(entity).and_then(|n| n.parent) {
            self.size_request(ldata, parent)
        } else {
            let w = ldata.5.width();
            let h = ldata.5.height();
            Vector2::new(w,h)
        }
    }

    fn update_layout(&mut self,ldata:&mut LayoutData,entity:Entity) {
        let cur_size:Vector2<f64> = self.size_request(ldata, entity);
        self.measure(ldata, entity,cur_size);
        self.arrange(ldata);
    }

    fn measure(&mut self,ldata:&mut LayoutData,entity:Entity,size:Vector2<f64>) -> Vector2<f64> {
        if let Some(stack) = ldata.4.get(entity) {
            stack.measure(entity, size,&mut ldata.6,&ldata.3,&ldata.2)
        } else if let Some(view) = ldata.3.get(entity) {
            view.measure(entity, size,&ldata.3,&mut ldata.6)
           
        } else {
            Vector2::zeros()
        }
    }
    fn arrange(&mut self,ldata:&mut LayoutData) {

    }

    fn is_invalid_measure(&self,entity:Entity) -> bool {
        true
    }
}

pub type LayoutData<'a> = (
    Entities<'a>,
    ReadExpect<'a,Tree>,
    ReadStorage<'a,TreeNode>,
    WriteStorage<'a,LayoutView>,
    WriteStorage<'a,StackLayout>,
    ReadExpect<'a,ViewPortSize>,
    WriteStorage<'a,Rect2D>);

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
           self.update_layout(&mut ldata,cur_entity);
       }
    }
}