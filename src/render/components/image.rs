use crate::assets::{Handle};
use specs::{Component,storage::{DenseVecStorage}};
use crate::render::types::*;
use crate::common::{Transform,rect::{Rect},Rect2D};
use crate::render::components::{ImageGenericInfo,ImageType,Mesh2D};
pub struct ImageRender {
    pub texture:Option<Handle<Texture>>,
    info:ImageGenericInfo,
}

impl Component for ImageRender {
    type Storage = DenseVecStorage<Self>;
}

impl ImageRender {
    pub fn new(texture:Option<Handle<Texture>>) -> ImageRender {
        ImageRender {
            texture,
            info: ImageGenericInfo {
                color: [1.0,1.0,1.0,1.0],
                typ: ImageType::Simple
            }
        }
    }

    pub fn set_texture(&mut self,tex:Handle<Texture>) {
        self.texture = Some(tex);
    }

    pub fn info_mut(&mut self) -> &mut ImageGenericInfo {
        &mut self.info
    }

    pub fn set_color(&mut self,r:f32,g:f32,b:f32,a:f32) {
        self.info.color[0] = r;
        self.info.color[1] = g;
        self.info.color[2] = b;
        self.info.color[3] = a;

    }

    pub fn get_color(&self) -> &[f32;4] {
        &self.info.color
    }

    pub fn set_type(&mut self,t:ImageType) {
        self.info.typ = t;
    }

    pub fn process_mesh(&mut self,trans:&Transform,raw_tex_size:(u32,u32),rect:&Rect2D,mesh2d:&mut Mesh2D) {
        if rect.width <= 0f32 && rect.height <= 0f32 {
            return
        }
        
        match mesh2d.mesh.as_mut() {
            Some(mesh) => {
                if mesh2d.is_dirty {
                    *mesh = self.info.to_mesh(trans,Rect {x:0f32,y:0f32,width:1f32,height:1f32},raw_tex_size,rect);
                    mesh2d.is_dirty = false;
                }
                let model:[[f32; 4]; 4] = (*trans.global_matrix()).into();
                mesh.sprite_arg.model = model.into();
            },
            None => {
                let mesh = self.info.to_mesh(trans,Rect {x:0f32,y:0f32,width:1f32,height:1f32},raw_tex_size,rect);
                mesh2d.mesh = Some(mesh);
                mesh2d.is_dirty = false;
            }
        }
    }
}