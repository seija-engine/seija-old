use specs::{Component,FlaggedStorage,DenseVecStorage};
use crate::common::{Transform};
use nalgebra::{Vector4,Vector2};

#[derive(Debug,Clone)]
pub struct Rect<T> {
    pub x:T,
    pub y:T,
    pub width:T,
    pub height:T
}

#[derive(Debug)]
pub struct Rect2D {
   pub width:f32,
   pub height:f32,
   pub anchor:[f32;2],
   pub dirty:bool
}

impl Default for Rect2D {
    fn default() -> Self {
        Rect2D {
            width:0f32,
            height:0f32,
            anchor:[0.5f32,0.5f32],
            dirty:false
        }
    }
}

impl Rect2D {
    pub fn new(w:f32,h:f32,anchor:[f32;2]) -> Self {
        Rect2D {
            width: w,
            height: h,
            anchor,
            dirty:false
        }
    }

    pub fn dirty(&mut self) {
        self.dirty = true
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn set_width(&mut self,width:f32) {
        if self.width != width {
            self.dirty()
        }
        self.width = width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn set_height(&mut self,height:f32) {
        if self.height != height {
            self.dirty()
        }
        self.height = height
    }

    pub fn anchor(&self) -> [f32;2] {
        self.anchor
    }

    pub fn set_anchor(&mut self,anchor:[f32;2]) {
        if self.anchor != anchor {
            self.dirty()
        }
        self.anchor = anchor
    }


    pub fn test(&self,t:&Transform,point:&(f64,f64)) -> bool {
        if self.width <= 0f32 || self.height <= 0f32 {
            return false;
        }
        let cx:f32 = point.0 as f32;
        let cy:f32 = point.1 as f32;
        let offset_x = self.width * self.anchor[0];
        let offset_y = self.height * self.anchor[1];
        let global = t.global_matrix();
        let pos = global.column(3).xyz();
        let x = - offset_x;
        let x1 = self.width - offset_x;
        let y =  self.height - offset_y;
        let y1 = - offset_y;
        let p0:Vector4<f32> = global * Vector4::new(x,y,pos.z,1f32);
        let p1:Vector4<f32> = global * Vector4::new(x1,y,pos.z,1f32);
        let p2:Vector4<f32> = global * Vector4::new(x,y1,pos.z,1f32);
        let am = Vector2::new(cx - p0.x ,cy - p0.y);
        let ab = Vector2::new(p1.x - p0.x,p1.y - p0.y);
        let ad = Vector2::new(p2.x - p0.x,p2.y - p0.y);
        let amdab = am.dot(&ab);
        let amdad = am.dot(&ad);
        if 0f32 <= am.dot(&ab) && amdab  <= ab.dot(&ab) && 0f32 <= amdad && amdad <= ad.dot(&ad) {
            return true
        }
        false
    }

    pub fn corner_point(&self) -> [f32;4] {
        let offset_x = -self.width * self.anchor[0];
        let offset_y = -self.height * self.anchor[1];
        [offset_x,offset_x + self.width,offset_y,offset_y + self.height]
    }

    pub fn left(&self) -> f32 {
        -self.width * self.anchor[0]
    }
    pub fn right(&self) -> f32 {
        self.width * (1f32 - self.anchor[0])
    }
    pub fn top(&self) -> f32 {
        self.height * (1f32 - self.anchor[0])
    }
    pub fn bottom(&self) -> f32 {
        -self.height * self.anchor[0]
    }
}

impl Component for Rect2D {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}