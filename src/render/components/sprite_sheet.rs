use crate::assets::{Handle};
use crate::common::rect::Rect;
use crate::render::types::{Texture};
use fnv::FnvHashMap;


#[derive(Debug, Clone)]
pub struct SpriteSheet {
    pub width: u32,
    pub height: u32,
    pub texture: Handle<Texture>,
    sprites: Vec<Sprite>,
    name_dic: FnvHashMap<String, u32>,
    borders: FnvHashMap<String, Vec<(f32, f32, f32, f32)>>,
}

impl SpriteSheet {
    pub fn new(
        width: u32,
        height: u32,
        texture: Handle<Texture>,
        sprites: Vec<Sprite>,
        name_dic: FnvHashMap<String, u32>,
        borders: FnvHashMap<String, Vec<(f32, f32, f32, f32)>>,
    ) -> Self {
        Self {
            width,
            height,
            texture,
            sprites,
            name_dic,
            borders,
        }
    }

    pub fn get_sprite(&self, sprite_name: &String) -> Option<&Sprite> {
        let idx = *self.name_dic.get(sprite_name)?;
        self.sprites.get(idx as usize)
    }

    pub fn get_border(&self, sprite_name: &String, idx: usize) -> Option<(f32, f32, f32, f32)> {
        let border_arr = self.borders.get(sprite_name)?;
        border_arr.get(idx).map(|v| *v)
    }
}

#[derive(Debug, Clone)]
pub struct Sprite {
    pub name: String,
    pub rect: Rect<i32>,
    pub coord: TextureCoordinate,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextureCoordinate {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
}