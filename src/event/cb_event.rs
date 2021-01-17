use specs::{Component,DenseVecStorage,World,WorldExt,Join,Entity,ReadStorage,WriteStorage};
use crate::common::{Rect2D,Transform,Hidden,HiddenPropagate,TreeNode};
use crate::event::{GameEvent,EventNode,NodeEvent,EventNodeState};
use std::sync::{Arc};

#[derive(Default)]
pub struct CABEventHandle;

impl CABEventHandle {
    pub fn process(&mut self,ev:&GameEvent,world:&mut World) {
       let mut events:Vec<(Arc<NodeEvent>,Entity)> = Vec::new();
       {
            let mut roots = world.write_storage::<CABEventRoot>();
            let trans = world.read_storage::<Transform>();
            let rects = world.read_storage::<Rect2D>();
            let hiddens = world.read_storage::<Hidden>();
            let hide_props = world.read_storage::<HiddenPropagate>();
            let mut ev_node = world.write_storage::<EventNode>();
            let tree_nodes = world.read_storage::<TreeNode>();

            for (e,root,t,rect,_,_) in (&world.entities(),&mut roots,&trans,&rects,!&hiddens,!&hide_props).join() {
                root.process(world,e,ev,t,rect,&tree_nodes,&trans,&rects,&mut ev_node,&mut events);   
            }
       };
       for (ev,eid) in events {
           ev(eid,world);
       }
    }
    
    pub fn process_no_hit(&mut self,ev:&GameEvent,world:&mut World) {
        let mut events:Vec<(Arc<NodeEvent>,Entity)> = Vec::new();
        {
            //let mut roots = world.write_storage::<CABEventRoot>();
            let trans = world.read_storage::<Transform>();
            let rects = world.read_storage::<Rect2D>();
            let hiddens = world.read_storage::<Hidden>();
            let hide_props = world.read_storage::<HiddenPropagate>();
            let mut ev_node = world.write_storage::<EventNode>();
            for (e,t,rect,ev_node,_,_) in (&world.entities(),&trans,&rects,&mut ev_node,!&hiddens,!&hide_props).join() {
                let pos = ev.get_pos();
                if rect.test(t, pos) == false {
                  if ev_node.node_state & EventNodeState::MouseIn.value() > 0 {
                     ev_node.node_state &= !(EventNodeState::MouseIn.value());
                     let ev_list = ev_node.get_dispatch_event(false,ev.to_type());
                     for ev in ev_list {
                        events.push((ev,e));
                     }
                  }
                }
            }
       };
       for (ev,eid) in events {
         ev(eid,world);
       }
    }
}

//捕获-冒泡方式的事件系统
#[derive(Default)]
pub struct CABEventRoot {
   
}

impl Component for CABEventRoot {
    type Storage = DenseVecStorage<CABEventRoot>;
}

impl CABEventRoot {
    pub fn process(&mut self,world:&World,e:Entity,ev:&GameEvent,trans:&Transform,rect:&Rect2D,
                   tree_nodes:&ReadStorage<TreeNode>,t_storage:&ReadStorage<Transform>,
                   r_storage:&ReadStorage<Rect2D>,ev_storage:&mut WriteStorage<EventNode>,events:&mut Vec<(Arc<NodeEvent>,Entity)>) {
        
        self.process_node(world,e, ev,trans,rect,tree_nodes,t_storage,r_storage,ev_storage,events);
        
        if let GameEvent::TouchEnd(pos) = ev {
            self.process_node(world,e, &GameEvent::Click(*pos),trans,rect,tree_nodes,t_storage,r_storage,ev_storage,events);
            for ev_node in ev_storage.join() {
                ev_node.node_state &= !(EventNodeState::MouseDown.value());
            }
        }
    }

  
    fn process_node(&mut self,world:&World,e:Entity,ev:&GameEvent,t:&Transform,rect:&Rect2D
                   ,tree_nodes:&ReadStorage<TreeNode>,t_storage:&ReadStorage<Transform>,
                   r_storage:&ReadStorage<Rect2D>,ev_storage:&mut WriteStorage<EventNode>,events:&mut Vec<(Arc<NodeEvent>,Entity)>) -> bool {
        let hide_props = world.read_storage::<HiddenPropagate>();
        if hide_props.contains(e) {
            return false;
        }
        let pos = ev.get_pos();
        if !rect.test(t, pos) {
            return false;
        }
        let zero_vec:Vec<Entity> = vec![];
        
        let mut ev_join = ev_storage.join();
        let may_ev_node = ev_join.get_unchecked(e.id());

        let hiddens = world.read_storage::<Hidden>();
        let is_hide = hiddens.contains(e);
        let mut is_through = false;
        //派发捕获事件 
        if let Some(ev_node) = may_ev_node {
            is_through = ev_node.is_through;
            if !is_hide && !ev_node.is_through {
                let evlist:Vec<Arc<NodeEvent>> = ev_node.get_dispatch_event(true,ev.to_type());
                for ev in evlist {
                    events.push((ev,e));
                }
                if ev_node.is_stop_capture {
                    return true;
                }
            }
        }

        let children:&Vec<Entity> = &tree_nodes.get(e).map(|t| &t.children).unwrap_or(&zero_vec);
        let is_last = children.len() == 0;
        
        //开始向上冒泡
        if is_last && !is_through {
            self.bubble_event(world,e,ev,tree_nodes,ev_storage,events);
        }

        let mut tr_joined = (t_storage,r_storage).join();
        for ce in children.iter().rev() {
            let may_get = tr_joined.get_unchecked(ce.id());
            if may_get.is_none() {
                continue;
            }
            let (t,rect) = may_get.unwrap();
            if self.process_node(world,*ce, ev, t,rect, tree_nodes,t_storage,r_storage,ev_storage,events) {
                return true;
            }
        }
        //if !is_last {
        //    self.bubble_event(world,e,ev,tree_nodes,ev_storage,events);
        //}
        
        return !is_through;
    }

    fn bubble_event(&mut self,world:&World,e:Entity,ev:&GameEvent,tree_nodes:&ReadStorage<TreeNode>,
                    ev_storage:&mut WriteStorage<EventNode>,events:&mut Vec<(Arc<NodeEvent>,Entity)>) {
        let is_hide = world.read_storage::<Hidden>().contains(e);
        
        let mut ev_join = ev_storage.join();
        if let Some(ev_node) = ev_join.get_unchecked(e.id()) {
            if is_hide == false {
                let evlist = ev_node.get_dispatch_event(false,ev.to_type());
                for ev in evlist {
                    events.push((ev,e));
                }
            }
            if ev_node.is_stop_bubble {
                return;
            }
        };
        let mut may_parent = tree_nodes.get(e).and_then(|p| p.parent);
        while let Some(parent) = may_parent {
            if let Some(ev_node) = ev_join.get_unchecked(parent.id()) {
                if is_hide == false {
                    let evlist = ev_node.get_dispatch_event(false,ev.to_type());
                    for ev in evlist {
                        events.push((ev,parent));
                    }
                };
                if ev_node.is_stop_bubble {
                    return;
                }
            };
            may_parent = tree_nodes.get(parent).unwrap().parent;
        }
    }

    
}