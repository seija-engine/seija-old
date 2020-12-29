mod image;
mod sprite;
mod sprite_sheet;
mod text;
mod mesh2d;
pub use image::{ImageRender};
pub use sprite::{SpriteRender};
pub use sprite_sheet::{SpriteSheet,Sprite,TextureCoordinate};
pub use text::{TextRender,LineMode};
pub use crate::render::SpriteMesh;
pub use mesh2d::{Mesh2D};
use crate::common::rect::{Rect};
use crate::common::{Transform,Rect2D};
use crate::render::pod::{SpriteArg,Vertex2D};

pub trait ISpriteMeshRender {
    fn to_sprite_mesh(&self,trans:&Transform,vert_start:u32,index_start:u16,rect:Option<Rect<f32>>,raw_size:(u32,u32)) -> SpriteMesh;
    fn is_dirty(&self) -> bool;
    fn clear_dirty(&mut self);
}

pub enum ImageType {
    Simple,
    Sliced(f32,f32,f32,f32), //left right top bottom
    Filled(ImageFilledType,f32),
    Tiled,
}

impl ImageType {
    pub fn cast_filled(&self) -> Option<(ImageFilledType,f32)> {
        match self {
            ImageType::Filled(ft,v) => Some((*ft,*v)),
            _ => None
        }
    }

    pub fn cast_sliced(&self) -> Option<(f32,f32,f32,f32)> {
        match self {
            ImageType::Sliced(l,r,t,b) => Some((*l,*r,*t,*b)),
            _ => None
        }
    }
}

#[derive(Copy,Clone,PartialEq,Eq)]
pub enum ImageFilledType {
    HorizontalLeft,
    HorizontalRight,
    VerticalTop,
    VerticalBottom
}

impl From<u32> for ImageFilledType {
    fn from(n: u32) -> ImageFilledType {
        match n {
            0 => ImageFilledType::HorizontalLeft,
            1 => ImageFilledType::HorizontalRight,
            2 => ImageFilledType::VerticalTop,
            _ => ImageFilledType::VerticalBottom,
        }
    }
}

pub struct ImageGenericInfo {
    //width:f32,
    //height:f32,
    pub typ:ImageType,
    //anchor:[f32;2],
    color:[f32;4],
}

impl ImageGenericInfo {
    pub fn to_mesh(&self,trans:&Transform,uv_rect:Rect<f32>,raw_size:(u32,u32),rect:&Rect2D) -> SpriteMesh {
        match self.typ {
            ImageType::Simple => self.to_simple_mesh(trans,uv_rect,rect),
            ImageType::Filled(_,_) => self.to_filled_mesh(trans,uv_rect,rect),
            ImageType::Sliced(_,_,_,_) => self.to_sliced_mesh(trans,uv_rect,raw_size,rect),
            _ => {
                unimplemented!()
            }
        }
    }

    pub fn set_fill_value(&mut self,val:f32) {
        if let ImageType::Filled(_,ref mut b) = self.typ {
            *b = val;
        }
    }

    pub fn mesh_by_quad(l:f32,r:f32,t:f32,b:f32,uv_l:f32,uv_r:f32,uv_t:f32,uv_b:f32,z:f32) -> Vec<Vertex2D> {
        vec![
              Vertex2D { //left top
                 pos: [l,t,z].into(),
                 uv:  [uv_l,uv_t].into()
              },
              Vertex2D { //right top
                pos: [r,t,z].into(),
                uv:  [uv_r,uv_t].into()
             },
             Vertex2D {//left bottom
                pos: [l,b,z].into(),
                uv:  [uv_l,uv_b].into()
             },
             Vertex2D {//right bottom
                pos: [r,b,z].into(),
                uv:  [uv_r,uv_b].into()
             },
        ]
    }

    fn quad_index(start:u16) -> Vec<u16> {
        vec![start + 0,start + 1,start + 2,start + 1,start + 3,start + 2]
    }

    pub fn to_simple_mesh(&self,trans:&Transform,uv_rect:Rect<f32>,rect:&Rect2D) -> SpriteMesh {
        let global = trans.global_matrix();
        let model:[[f32; 4]; 4] = (*trans.global_matrix()).into();
        let sprite_arg = SpriteArg {
            model: model.into(),
            color: self.color.into()
        };
        let col3 =  global.column(3);
        let z = col3[2];
        let offset_x = -rect.width() * rect.anchor()[0];
        let offset_y = -rect.height() * rect.anchor()[1];
        let meshes = vec![
              Vertex2D { //left top
                 pos: [0f32 + offset_x,rect.height() + offset_y,z].into(),
                 uv:  [uv_rect.x,uv_rect.y].into()
              },
              Vertex2D { //right top
                pos: [rect.width() + offset_x,rect.height() + offset_y,z].into(),
                uv:  [uv_rect.x + uv_rect.width,uv_rect.y].into()
             },
             Vertex2D {//left bottom
                pos: [0f32 + offset_x,0f32 + offset_y,z].into(),
                uv:  [uv_rect.x,uv_rect.y + uv_rect.height].into()
             },
             Vertex2D {//right bottom
                pos: [rect.width() + offset_x,0f32 + offset_y,z].into(),
                uv:  [uv_rect.x + uv_rect.width,uv_rect.y + uv_rect.height].into()
             },
        ];
        SpriteMesh {
            sprite_arg,
            meshes,
            indexs:vec![0,1,2,1,3,2]
        }
    }

    pub fn to_sliced_mesh(&self,trans:&Transform,uv_rect:Rect<f32>,raw_size:(u32,u32),rect:&Rect2D) -> SpriteMesh {
        let global = trans.global_matrix();
        let model:[[f32; 4]; 4] = (*trans.global_matrix()).into();
        let sprite_arg = SpriteArg {
            model: model.into(),
            color: self.color.into()
        };
        let col3 =  global.column(3);
        let z = col3[2];
        let (border_left,border_right,border_top,border_bottom) = self.typ.cast_sliced().unwrap();
        let offset_x = rect.width() * rect.anchor()[0];
        let offset_y = rect.height() * rect.anchor()[1];
        let (x0,x1,x2,x3) = (- offset_x,border_left - offset_x,rect.width() - border_right - offset_x,rect.width() - offset_x);
        let (y0,y1,y2,y3) = (rect.height() - offset_y,rect.height() - border_top - offset_y,border_bottom - offset_y,- offset_y);
        
        
        let  left_uv_width  = uv_rect.width * (border_left / raw_size.0 as f32);
        let  right_uv_width = uv_rect.width * (border_right / raw_size.0 as f32);
        let  top_uv_height = uv_rect.height * (border_top / raw_size.1 as f32);
        let  bottom_uv_height = uv_rect.height * (border_bottom / raw_size.1 as f32);
        
        let (uv_x0,uv_x1,uv_x2,uv_x3) = (uv_rect.x,uv_rect.x + left_uv_width,uv_rect.x + uv_rect.width - right_uv_width,uv_rect.x + uv_rect.width);
        let (uv_y0,uv_y1,uv_y2,uv_y3) = (uv_rect.y,uv_rect.y + top_uv_height,uv_rect.y + uv_rect.height - bottom_uv_height,uv_rect.y + uv_rect.height);
        
        let mut ia = 0;
        //center
        let mut meshes:Vec<Vertex2D> = ImageGenericInfo::mesh_by_quad(x1,x2,y1,y2,uv_x1,uv_x2,uv_y1,uv_y2,z);
        let mut indexs:Vec<u16> = ImageGenericInfo::quad_index(ia);
         //left top
        if border_left > 0f32 && border_top > 0f32 {
            
            meshes.extend(ImageGenericInfo::mesh_by_quad(x0,x1,y0,y1,uv_x0,uv_x1,uv_y0,uv_y1,z));
            ia += 4;
            indexs.extend(ImageGenericInfo::quad_index(ia));
        }
       
        //top
        if border_left > 0f32 && border_top > 0f32 {
            meshes.extend(ImageGenericInfo::mesh_by_quad(x1,x2,y0,y1,uv_x1,uv_x2,uv_y0,uv_y1,z));
            ia += 4;
            indexs.extend(ImageGenericInfo::quad_index(ia));
        }
        //right top
        if border_left > 0f32 && border_top > 0f32 {
            meshes.extend(ImageGenericInfo::mesh_by_quad(x2,x3,y0,y1,uv_x2,uv_x3,uv_y0,uv_y1,z));
            ia += 4;
            indexs.extend(ImageGenericInfo::quad_index(ia));
        }
        //left
        if border_left > 0f32 && border_top > 0f32 {
            meshes.extend(ImageGenericInfo::mesh_by_quad(x0,x1,y1,y2,uv_x0,uv_x1,uv_y1,uv_y2,z));
            ia += 4;
            indexs.extend(ImageGenericInfo::quad_index(ia));
        }
        //right
        if border_left > 0f32 && border_top > 0f32 {
            meshes.extend(ImageGenericInfo::mesh_by_quad(x2,x3,y1,y2,uv_x2,uv_x3,uv_y1,uv_y2,z));
            ia += 4;
            indexs.extend(ImageGenericInfo::quad_index(ia));
        }
        //bottom left
        if border_left > 0f32 && border_top > 0f32 {
            meshes.extend(ImageGenericInfo::mesh_by_quad(x0,x1,y2,y3,uv_x0,uv_x1,uv_y2,uv_y3,z));
            ia += 4;
            indexs.extend(ImageGenericInfo::quad_index(ia));
        }
        //bottom
        if border_left > 0f32 && border_top > 0f32 {
            meshes.extend(ImageGenericInfo::mesh_by_quad(x1,x2,y2,y3,uv_x1,uv_x2,uv_y2,uv_y3,z));
            ia += 4;
            indexs.extend(ImageGenericInfo::quad_index(ia));
        }
        //bottom right
        if border_left > 0f32 && border_top > 0f32 {
            meshes.extend(ImageGenericInfo::mesh_by_quad(x2,x3,y2,y3,uv_x2,uv_x3,uv_y2,uv_y3,z));
            ia += 4;
            indexs.extend(ImageGenericInfo::quad_index(ia));
        }
        SpriteMesh {
            sprite_arg,
            meshes,
            indexs
        }
    }

    pub fn to_filled_mesh(&self,trans:&Transform,uv_rect:Rect<f32>,rect:&Rect2D) -> SpriteMesh {
        let global = trans.global_matrix();
        let model:[[f32; 4]; 4] = (*trans.global_matrix()).into();
        let sprite_arg = SpriteArg {
            model: model.into(),
            color: self.color.into()
        };
        let col3 =  global.column(3);
        let z = col3[2];

        let (fill_typ,fill_val) = self.typ.cast_filled().unwrap();

        let anchor_offset_x = -rect.width() * rect.anchor()[0];
        let anchor_offset_y = -rect.height() * rect.anchor()[1];

        let mut left = 0f32 + anchor_offset_x;
        let mut right = rect.width() + anchor_offset_x;
        let mut top = rect.height() + anchor_offset_y;
        let mut bottom = 0f32 + anchor_offset_y;
        let mut uv_left = uv_rect.x;
        let mut uv_right = uv_rect.x + uv_rect.width;
        let mut uv_top = uv_rect.y;
        let mut uv_bottom = uv_rect.y + uv_rect.height;

        let sub_uv = 1f32 - fill_val;
        match fill_typ {
            ImageFilledType::HorizontalLeft => {
                let sub_val = rect.width() - rect.width() * fill_val;
                right = right - sub_val;
                uv_right = uv_right - (uv_rect.width * sub_uv);
            },
            ImageFilledType::HorizontalRight => {
                let sub_val = rect.width() - rect.width() * fill_val;
                left = left + sub_val;
                uv_left = uv_left + (uv_rect.width * sub_uv);
            },
            ImageFilledType::VerticalTop => {
                let sub_val = rect.height() - rect.height() * fill_val;
                bottom = bottom + sub_val;
                uv_bottom = uv_bottom - (uv_rect.height * sub_uv);
            },
            ImageFilledType::VerticalBottom => {
                let sub_val = rect.height() - rect.height() * fill_val;
                top = top - sub_val;
                uv_top = uv_top + (uv_rect.height * sub_uv);
            },
        };

        SpriteMesh {
            sprite_arg,
            meshes: ImageGenericInfo::mesh_by_quad(left,right,top,bottom,uv_left,uv_right,uv_top,uv_bottom,z),
            indexs:vec![0,1,2,1,3,2]
        }
    }
}

