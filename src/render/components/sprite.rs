use crate::render::components::{SpriteSheet,Mesh2D};
use crate::assets::{Handle,AssetStorage};
use crate::common::{Transform,rect::{Rect},Rect2D};
use specs::{Component,storage::{DenseVecStorage}};
use crate::render::components::{ImageGenericInfo,ImageType,Sprite};


pub struct SpriteRender {
    info:ImageGenericInfo,
    sprite_name:Option<String>,
    pub sprite_sheet:Option<Handle<SpriteSheet>>,
}

impl SpriteRender {
    pub fn new(sprite_sheet:Option<Handle<SpriteSheet>>,sprite_name:Option<&str>) -> Self {
        SpriteRender {
            sprite_name: sprite_name.map(|s| s.to_string()),
            sprite_sheet,
            info: ImageGenericInfo {
                color: [1.0,1.0,1.0,1.0],
                typ: ImageType::Simple,
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        self.sprite_name.is_some() && self.sprite_sheet.is_some()
    }

    /*
    pub fn set_native_size(&mut self,sprite_sheet_storage:&AssetStorage<SpriteSheet>) {
       let sprite_sheet = sprite_sheet_storage.get(&self.sprite_sheet).unwrap();
       let sprite = sprite_sheet.get_sprite(&self.sprite_name).unwrap();
       self.info.width =  sprite.rect.width as f32;
       self.info.height = sprite.rect.height as f32;
    }*/

    pub fn get_sprite(&self,sprite_sheet_storage:&AssetStorage<SpriteSheet>) -> Option<Sprite> {
        if self.is_valid() {
            let sprite_sheet = sprite_sheet_storage.get(self.sprite_sheet.as_ref().unwrap()).unwrap();
            sprite_sheet.get_sprite(&self.sprite_name.as_ref().unwrap()).map(|s| s.clone())
        } else {
            None
        }
    }

    pub fn set_sprite_name(&mut self,sprite_name:Option<&str>) {
        self.sprite_name = sprite_name.map(|s|s.to_string());
    }

    pub fn set_type(&mut self,typ:ImageType) {
        self.info.typ = typ;
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

    pub fn info_mut(&mut self) -> &mut ImageGenericInfo {
        &mut self.info
    }

    pub fn set_slice_type_by_cfg(&mut self,idx:usize,sheet_storage:&AssetStorage<SpriteSheet>) {
        if !self.is_valid() {
            return;
        }
        let border = sheet_storage.get(self.sprite_sheet.as_ref().unwrap())
                                                              .and_then(|s| s.get_border(self.sprite_name.as_ref().unwrap(), idx));
        if let Some((_0,_1,_2,_3)) = border {
            self.info.typ = ImageType::Sliced(_0,_1,_2,_3);
        }
    }

    pub fn set_fill_value(&mut self,val:f32) {
        if let ImageType::Filled(_,ref mut b) = self.info.typ {
            *b = val;
        }
    }

    pub fn uv_rect_and_size(&self,storage:&AssetStorage<SpriteSheet>) -> Option<(Rect<f32>,(u32,u32))> {
        let sprite_sheet = storage.get(&self.sprite_sheet.as_ref().unwrap()).unwrap();
        let sprite = sprite_sheet.get_sprite(&self.sprite_name.as_ref().unwrap()).unwrap();
        Some((Rect {
            x: sprite.coord.left,
            y: sprite.coord.top,
            width: sprite.coord.right - sprite.coord.left,
            height: sprite.coord.bottom - sprite.coord.top
        },(sprite.rect.width as u32,sprite.rect.height as u32)))
    }

    pub fn sprite_name(&self) -> Option<&String> {
        self.sprite_name.as_ref()
    }

    pub fn set_sprite_sheet(&mut self,sheet:Option<Handle<SpriteSheet>>) {
        self.sprite_sheet = sheet;
    }

    pub fn process_mesh(&mut self,t:&Transform,storage:&AssetStorage<SpriteSheet>,rect2d:&Rect2D,mesh2d:&mut Mesh2D) {
        if ! self.is_valid() {
            return;
        }
        match mesh2d.mesh.as_mut() {
            Some(mesh) => {
                if mesh2d.is_dirty {
                    let (rect,size) = self.uv_rect_and_size(&storage).unwrap();
                    let new_mesh = self.info.to_mesh(t,rect,size,rect2d);
                    *mesh = new_mesh;
                    mesh2d.is_dirty = false;
                }
                let model:[[f32; 4]; 4] = (*t.global_matrix()).into();
                mesh.sprite_arg.model = model.into();
            },
            None => {
                let (rect,size) = self.uv_rect_and_size(&storage).unwrap();
                let new_mesh = self.info.to_mesh(t,rect,size,rect2d);
                mesh2d.mesh = Some(new_mesh);
                mesh2d.is_dirty = false;
            }
        }
    }
}

impl Component for SpriteRender {
    type Storage = DenseVecStorage<Self>;
}