use winit::{event::{Event,WindowEvent,ElementState}};
use specs::{World,Entity,WorldExt,Component,DenseVecStorage,Join};
pub mod cb_event;
pub mod global;
use crate::event::cb_event::{CABEventHandle,CABEventRoot};
use std::collections::{HashMap};
use std::sync::{Arc};
#[derive(Debug,Clone)]
pub enum GameEvent {
    TouchStart((f64,f64)),
    TouchEnd((f64,f64)),
    Click((f64,f64)),
    Move((f64,f64)),
    MouseEnter((f64,f64)),
    MouseLeave((f64,f64)),
    KeyBoard(u32,bool)
}

pub trait GameEventCallBack  :Send + Sync{
    fn run(&self,ev:&GameEvent);
}

#[derive(PartialEq,Eq,Hash,Clone,Debug)]
pub enum GameEventType {
    TouchStart = 0,
    TouchEnd = 1,
    Click = 2,
    MouseMove = 3,
    MouseEnter = 4,
    MouseLeave = 5,
    KeyBoard = 6
}

impl GameEventType {
    pub fn from(t:u32) -> Option<GameEventType> {
        match t {
            0 => Some(GameEventType::TouchStart),
            1 => Some(GameEventType::TouchEnd),
            2 => Some(GameEventType::Click),
            3 => Some(GameEventType::MouseMove),
            4 => Some(GameEventType::MouseEnter),
            5 => Some(GameEventType::MouseLeave),
            6 => Some(GameEventType::KeyBoard),
            _ => None
        }
    }
}

impl GameEvent {
    pub fn to_type(&self) -> GameEventType {
        match self {
            GameEvent::TouchStart(_) => GameEventType::TouchStart,
            GameEvent::TouchEnd(_) => GameEventType::TouchEnd,
            GameEvent::Click(_) => GameEventType::Click,
            GameEvent::Move(_) => GameEventType::MouseMove,
            GameEvent::MouseEnter(_) => GameEventType::MouseEnter,
            GameEvent::MouseLeave(_) => GameEventType::MouseLeave,
            GameEvent::KeyBoard(_,_) => GameEventType::KeyBoard,
        }
    }

    pub fn get_pos(&self) -> &(f64,f64) {
        match self {
            GameEvent::TouchStart(pos) => pos,
            GameEvent::TouchEnd(pos) => pos,
            GameEvent::Click(pos) => pos,
            GameEvent::Move(pos) => pos,
            GameEvent::MouseEnter(pos) => pos,
            GameEvent::MouseLeave(pos) => pos,
            _ => &(0f64,0f64)
        }
    }

}

type NodeEvent = Box<dyn Fn(Entity,&World) + 'static + Send + Sync>;

pub enum EventNodeState {
    MouseDown,
    MouseIn,
}

impl EventNodeState {
    fn value(self) -> u32 {
        match self {
            EventNodeState::MouseDown => 0x01,
            EventNodeState::MouseIn => 0x02
        }
    }
}

#[derive(Default,Clone)]
pub struct EventNode {
    pub node_state:u32,
    capture_event:HashMap<GameEventType,Arc<NodeEvent>>,
    bubble_event:HashMap<GameEventType,Arc<NodeEvent>>,
    is_stop_capture:bool,
    is_stop_bubble:bool
}

impl EventNode {

    pub fn register<F>(&mut self,is_capture:bool,typ:GameEventType,f:F) where F:Fn(Entity,&World) + 'static + Send + Sync  {
        if is_capture {
            self.capture_event.insert(typ,Arc::new(Box::new(f)));
        } else {
            self.bubble_event.insert(typ,Arc::new(Box::new(f)));
        };
    }

    pub fn get_dispatch_event(&mut self,is_capture:bool,ev_type:GameEventType) -> Vec<Arc<NodeEvent>> {
        let may_node_ev = self.get_event_by_type(is_capture, &ev_type).map(|a| a.clone());
        let mut ret_events:Vec<Arc<NodeEvent>> = vec![];
        match ev_type {
            GameEventType::TouchStart => {
                self.node_state |= EventNodeState::MouseDown.value();
                if let Some(node_ev) = may_node_ev {
                    ret_events.push(node_ev);
                }
            },
            GameEventType::MouseEnter => {
                if self.node_state & EventNodeState::MouseIn.value() == 0 {
                    if let Some(node_ev) = may_node_ev {
                        ret_events.push(node_ev);
                    }
                    if is_capture == false {
                        self.node_state |= EventNodeState::MouseIn.value();
                    }
                }
            },
            GameEventType::Click => {
                if self.node_state & EventNodeState::MouseDown.value() > 0 {
                    if let Some(node_ev) = may_node_ev {
                        ret_events.push(node_ev);
                    }
                }
            },
            _ => {
                if let Some(node_ev) = may_node_ev {
                    ret_events.push(node_ev);
                }
            }
        };
        
        return ret_events;
    }

   pub fn get_event_by_type(&self,is_capture:bool,ev_type:&GameEventType) -> Option<&Arc<NodeEvent>> {
      if is_capture {
        self.capture_event.get(ev_type)
    } else {
        self.bubble_event.get(ev_type)
    }
   }
}

impl Component for EventNode {
    type Storage = DenseVecStorage<EventNode>;
}

pub struct GameEventHandle {
    mouse_pos:(f64,f64),
    cab_event_handle:CABEventHandle,
    view_size:(f64,f64),
}


impl GameEventHandle {
    pub fn new() -> Self {
        GameEventHandle {
            mouse_pos: (0f64,0f64),
            cab_event_handle: CABEventHandle {},
            view_size:(0f64,0f64)
        }
    }

    pub fn set_view_size(&mut self,size:(f64,f64)) {
        self.view_size = size;
    }

    pub fn register(world:&mut World) {
        world.register::<CABEventRoot>();
        world.register::<EventNode>();
        world.register::<global::GlobalEventNode>();
    }

    fn conv_pos(&self,x:f64,y:f64) -> (f64,f64) {
        (x - self.view_size.0 * 0.5f64,-(y - self.view_size.1 * 0.5f64))
    }

    pub fn fire_event(&mut self,events:&Vec<Event<()>>,world:&mut World) {
        let _a = &self.cab_event_handle;
        for ev in events.iter() {
            match  ev {
                Event::WindowEvent  {event,..} => {
                    match event {
                        WindowEvent::Touch(d) => {
                            dbg!(&d);
                        },
                        WindowEvent::MouseInput {state,..} => {
                           
                           if *state == ElementState::Pressed {
                               self.cab_event_handle.process(&GameEvent::TouchStart(self.mouse_pos),world);
                           } else {
                               self.cab_event_handle.process(&GameEvent::TouchEnd(self.mouse_pos),world);
                           }
                        },
                        WindowEvent::CursorMoved {position,..} => {
                            self.mouse_pos = self.conv_pos(position.x as f64, position.y as f64);
                            self.cab_event_handle.process(&GameEvent::Move(self.mouse_pos) , world);
                            self.cab_event_handle.process(&GameEvent::MouseEnter(self.mouse_pos) , world);
                            self.cab_event_handle.process_no_hit(&GameEvent::MouseLeave(self.mouse_pos) , world);
                        },
                        WindowEvent::KeyboardInput{input,..} => {
                            let calls = GameEventHandle::get_global_calls(world, GameEventType::KeyBoard);
                            for ev in calls.iter() {
                                let code = input.virtual_keycode.map(|v|v as u32).unwrap_or(0u32);
                                let game_ev = GameEvent::KeyBoard(code,input.state == ElementState::Pressed);
                                ev.run(&game_ev);
                            }
                        }
                        _ => ()
                    }
                },
                _ => ()
            }
        }
    }

    fn get_global_calls(world:&World,typ:GameEventType) -> Vec<Arc<Box<dyn GameEventCallBack>>> {
        let mut ret_vec  = vec![];
        let global_evs = world.read_storage::<global::GlobalEventNode>();
        for ev in global_evs.join() {
           if let Some(call) = ev.get_rc(&typ) {
               ret_vec.push(call);
           }
        }
        ret_vec
    }
}