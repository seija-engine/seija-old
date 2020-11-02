use specs::{Component,VecStorage,World,WorldExt,Join};
use std::sync::{Arc,Mutex};
pub enum UpdateType {
    Frame(u32),
    Time(f32)
}
pub trait UpdateCallBack :Send + Sync {
    fn run(&self);
}

#[derive(Default)]
pub struct Update {
    updates:Vec<Arc<UpdateDesc>>
}

impl Update {
    pub fn update(&self,dt:f32) {
        for d in self.updates.iter() {
            d.update(dt);
        }
    }

    pub fn insert(&mut self,desc:UpdateDesc) {
        self.updates.push(Arc::new(desc));
    }
}

pub struct UpdateDesc {
    typ:UpdateType,
    call:Option<Box<dyn UpdateCallBack>>,
    add_frame:Mutex<u32>,
    add_time:Mutex<f32>
}

impl UpdateDesc {
    pub fn update(&self,dt:f32) {
        match self.typ {
            UpdateType::Frame(frame) => {
                let mut mutframe = self.add_frame.lock().unwrap();
                if *mutframe >= frame {
                    if let Some(f) = self.call.as_ref() {
                        f.run();
                    }
                    *mutframe = 1;
                } else {
                    *mutframe += 1;
                }
            },
            UpdateType::Time(time) => {
                let mut muttime = self.add_time.lock().unwrap();
                if *muttime >= time {
                    if let Some(f) = self.call.as_ref() {
                        f.run();
                    }
                    *muttime = dt;
                } else {
                    *muttime += dt;
                }
            },
        }
    }

    pub fn from_frame(num:u32) -> Self {
        UpdateDesc {typ:UpdateType::Frame(num),call:None,add_frame:Mutex::new(0),add_time:Mutex::new(0f32) }
    }

    pub fn from_time(num:f32) -> Self {
        UpdateDesc {typ:UpdateType::Time(num),call:None ,add_frame:Mutex::new(0),add_time:Mutex::new(0f32)}
    }

    pub fn set_call(&mut self,call:Box<dyn UpdateCallBack>) {
        self.call = Some(call);
    }
}

impl Component for Update {
    type Storage = VecStorage<Self>;
}


pub struct UpdateSystem {
    update_desc:Vec<Arc<UpdateDesc>>
}

impl Default for UpdateSystem {
    fn default() -> Self {
        UpdateSystem {
            update_desc:vec![]
        }
    }
}

impl UpdateSystem {
    pub fn update(&mut self,dt:f32,world:&mut World) {
       self.update_desc.clear();
       {
           let mut storage = world.write_storage::<Update>();
           let joined = (&mut storage).join();
           for update in joined {
               for desc in update.updates.iter() {
                   self.update_desc.push(desc.clone());
               } 
             
           }
        };
        for desc in self.update_desc.iter() {
            desc.update(dt);
        }
    }
}