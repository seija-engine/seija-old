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
    pub anchor:[f32;2]
}

impl Default for Rect2D {
    fn default() -> Self {
        Rect2D {
            width:0f32,
            height:0f32,
            anchor:[0.5f32,0.5f32]
        }
    }
}

impl Rect2D {
    pub fn new(w:f32,h:f32,anchor:[f32;2]) -> Self {
        Rect2D {
            width: w,
            height: h,
            anchor
        }
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
}

impl Component for Rect2D {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}