use specs::{Component,VecStorage};
use crate::common::{Transform,Rect2D};
use crate::window::{ViewPortSize};
use nalgebra::{Vector3};
pub enum ScaleType {
    ScaleWithWidth(f32),
    ScaleWithHeight(f32)
}
pub struct ScreenScaler {
    typ:ScaleType,
    dirty:bool
}

impl Component for ScreenScaler {
    type Storage = VecStorage<ScreenScaler>;
}

impl ScreenScaler {
    pub fn with_scale_width(width:f32) -> Self {
        ScreenScaler {
            typ:ScaleType::ScaleWithWidth(width),
            dirty:true
        }
    }

    pub fn with_scale_height(height:f32) -> Self {
        ScreenScaler {
            typ:ScaleType::ScaleWithHeight(height),
            dirty:true
        }
    }

    pub fn set_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }
    
    pub fn process(&mut self,port_size:&ViewPortSize,t:&mut Transform,rect:&mut Rect2D) {
       if self.dirty == false {
           return;
        }
        self.dirty = false;
        match self.typ {
            ScaleType::ScaleWithWidth(vw)  => {
                let scale = port_size.width() as f32 / vw;
                t.set_scale(Vector3::new(scale,scale,1f32));
                rect.width = vw;
                rect.height = (vw / port_size.width() as f32) * (port_size.height()  as f32);
            },
            ScaleType::ScaleWithHeight(vh) => {
                let scale =  port_size.height() as f32 / vh;
                t.set_scale(Vector3::new(scale,scale,1f32));
                rect.width = (vh / port_size.height() as f32) * (port_size.width()  as f32);
                rect.height = vh;
            }
        }
    }
}