use crate::{common::{Tree,TreeNode,TreeEvent}, window::ViewPortSize};
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
        if !size.is_empty() {
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

    fn update_layout(&mut self,ldata:&LayoutData,entity:Entity) {
        let cur_size:Vector2<f64> = self.size_request(ldata, entity);
        self.measure(ldata, entity,cur_size);
        self.arrange(ldata);
    }

    fn measure(&mut self,ldata:&LayoutData,entity:Entity,size:Vector2<f64>) -> Vector2<f64> {
        if let Some(stack) = ldata.4.get(entity) {
            stack.measure(ldata, size)
        } else if let Some(view) = ldata.3.get(entity){
            view.measure(ldata, size)
        } else {
            Vector2::zeros()
        }
    }
    fn arrange(&mut self,ldata:&LayoutData) {

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
    ReadExpect<'a,ViewPortSize>);

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
    
    fn run(&mut self, ldata: Self::SystemData) {
       self.modified.clear();
       let tree = &ldata.1;
       let tree_nodes = &ldata.2;
       let entities = &ldata.0;
       for ev in tree.channel.read(&mut self.ev_tree) {
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
           let cur_entity = entities.entity(eid);
           self.update_layout(&ldata,cur_entity);
       }
    }
}