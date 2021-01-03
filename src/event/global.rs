use crate::event::{GameEventCallBack,GameEventType,GameEvent};
use std::collections::HashMap;
use specs::{Component,VecStorage};
use std::sync::{Arc};

#[derive(Default)]
pub struct GlobalEventNode {
    events:HashMap<GameEventType,Arc<Box<dyn GameEventCallBack>>>
}

impl GlobalEventNode {
   

    pub fn insert(&mut self,typ:GameEventType,call:Box<dyn GameEventCallBack>) {
        self.events.insert(typ, Arc::new(call));
    }

    pub fn get_rc(&self,typ:&GameEventType) -> Option<Arc<Box<dyn GameEventCallBack>>> {
        self.events.get(typ).map(|v| v.clone())
    }
}

impl Component for GlobalEventNode {
    type Storage = VecStorage<Self>;
}
