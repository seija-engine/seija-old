use crate::common::Rect2D;

use super::types::{Thickness,LayoutAlignment};
use specs::{Component, Entity, FlaggedStorage, ReadStorage, WriteStorage};
use nalgebra::Vector2;
use super::LayoutData;

#[derive(Clone,Default)]
pub struct LayoutView {
    pub force_size:Vector2<f64>,
    pub size:Vector2<f64>,
    pub margin:Thickness,
    pub padding:Thickness,
    pub hor:LayoutAlignment,
    pub ver:LayoutAlignment
}

impl Component for LayoutView {
    type Storage = FlaggedStorage<LayoutView>;
}

impl LayoutView {
    pub fn measure(&self,entity:Entity,size:Vector2<f64>,
                   views:&WriteStorage<LayoutView>,
                   rect2ds:&mut WriteStorage<Rect2D>) -> Vector2<f64> {
        let view = views.get(entity).unwrap();
        let rect2d = rect2ds.get(entity).unwrap();
        let mut w:f64 = rect2d.width as f64;
        let mut h:f64 = rect2d.height as f64;
        if view.hor == LayoutAlignment::Fill {
            w = size.x - self.margin.horizontal();
        }
        if view.ver == LayoutAlignment::Fill {
            h = size.y - self.margin.vertical();
        }

        let rect2d = rect2ds.get_mut(entity).unwrap();
        rect2d.width = w as f32;
        rect2d.height = h as f32;
        Vector2::new(w,h)
    }
}