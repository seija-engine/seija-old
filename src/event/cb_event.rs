use specs::{Component,DenseVecStorage,World,WorldExt,Join,Entity,ReadStorage,WriteStorage};
use crate::common::{Rect2D,Transform,transform::{ParentHierarchy},Hidden,HiddenPropagate};
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
            let hierarchy = world.fetch::<ParentHierarchy>();

            for (e,root,t,rect,_,_) in (&world.entities(),&mut roots,&trans,&rects,!&hiddens,!&hide_props).join() {
                root.process(world,e,ev,t,rect,&hierarchy,&trans,&rects,&mut ev_node,&mut events);   
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
                   hierarchy:&ParentHierarchy,t_storage:&ReadStorage<Transform>,
                   r_storage:&ReadStorage<Rect2D>,ev_storage:&mut WriteStorage<EventNode>,events:&mut Vec<(Arc<NodeEvent>,Entity)>) {
        
        self.process_node(world,e, ev,trans,rect,hierarchy,t_storage,r_storage,ev_storage,events);
        
        if let GameEvent::TouchEnd(pos) = ev {
            self.process_node(world,e, &GameEvent::Click(*pos),trans,rect,hierarchy,t_storage,r_storage,ev_storage,events);
            for ev_node in ev_storage.join() {
                ev_node.node_state &= !(EventNodeState::MouseDown.value());
            }
        }
    }

  
    fn process_node(&mut self,world:&World,e:Entity,ev:&GameEvent,t:&Transform,rect:&Rect2D
                   ,hierarchy:&ParentHierarchy,t_storage:&ReadStorage<Transform>,
                   r_storage:&ReadStorage<Rect2D>,ev_storage:&mut WriteStorage<EventNode>,events:&mut Vec<(Arc<NodeEvent>,Entity)>) -> bool {
        let hide_props = world.read_storage::<HiddenPropagate>();
        if hide_props.contains(e) {
            return false;
        }
        let pos = ev.get_pos();
        if rect.test(t, pos) == false {
            return false;
        }
        let children = hierarchy.children(e);
        let is_last = children.len() == 0;
        let mut ev_join = ev_storage.join();
        let may_ev_node = ev_join.get_unchecked(e.id());

        let hiddens = world.read_storage::<Hidden>();
        let is_hide = hiddens.contains(e);
        
        //派发捕获事件
        if may_ev_node.is_some() && is_last == false &&  is_hide == false {
            let ev_node = may_ev_node.unwrap();
            let evlist = ev_node.get_dispatch_event(true,ev.to_type());
            for ev in evlist {
                events.push((ev,e));
            }
            if ev_node.is_stop_capture {
                return true;
            }
        };

        //开始向上冒泡
        if is_last {
            self.bubble_event(world,e,ev,hierarchy,ev_storage,events);
        }
        let mut tr_joined = (t_storage,r_storage).join();
        for ce in children {
            let may_get = tr_joined.get_unchecked(ce.id());
            if may_get.is_none() {
                continue;
            }
            let (t,rect) = may_get.unwrap();
            if self.process_node(world,*ce, ev, t,rect, hierarchy,t_storage,r_storage,ev_storage,events) {
                return true;
            }
        }
        if is_last == false {
            self.bubble_event(world,e,ev,hierarchy,ev_storage,events);
        }
        return true;
    }

    fn bubble_event(&mut self,world:&World,e:Entity,ev:&GameEvent,hierarchy:&ParentHierarchy,
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
        let mut may_parent = hierarchy.parent(e);
        while may_parent.is_some() {
            let parent = may_parent.unwrap();
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
            may_parent = hierarchy.parent(parent);
        }
    }

    
}