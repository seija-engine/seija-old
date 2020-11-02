use rendy::texture::{image::{ImageTextureConfig,load_from_image},TextureBuilder};
use crate::render::types::{Texture,Backend};
use crate::assets::{IAssetLoaderInfo,LoaderEnv,AssetLoadError,StorageCenter,Handle,Asset};
use rendy::factory::{Factory,ImageState};
use rendy::command::{QueueId};
use crate::common::rect::{Rect};
use std::path::{Path};
use crate::render::components::{SpriteSheet,Sprite,TextureCoordinate};
use fnv::FnvHashMap;
use crate::render::{FontAsset};
use specs::{World};
use glyph_brush::rusttype::{Font};

pub struct TextuteLoaderInfo {
    path:String,
    config:ImageTextureConfig
}

impl TextuteLoaderInfo {
    pub fn new(path:&str,config:ImageTextureConfig) -> TextuteLoaderInfo {
        TextuteLoaderInfo {path : String::from(path),config }
    }

    pub fn new_only_path(path:&str) -> TextuteLoaderInfo {
        TextuteLoaderInfo {path :String::from(path),config :ImageTextureConfig::default() }
    }
}

impl Asset for Texture {
    type LoaderInfo = TextuteLoaderInfo;
}

impl IAssetLoaderInfo for TextuteLoaderInfo {
    type CData = TextureBuilder<'static>;
    type Asset = Texture;

    fn path(&self) -> &String {
        &self.path
    }

    fn load_data(&self,_:&StorageCenter,source:&LoaderEnv) -> Result<Result<Self::CData,Self::Asset>,AssetLoadError> {
        let bytes = source.load_fs_source(self.path.as_str())?;
        let builder:TextureBuilder = load_from_image(std::io::Cursor::new(bytes), self.config.clone())
                                     .map_err(|_| AssetLoadError::LoadImageError)?;
        Ok(Ok(builder))
    }

    fn load<B:Backend>(builder:TextureBuilder<'static>,factory:&mut Factory<B>,qid:QueueId,_:&StorageCenter,_:&World) -> Result<Self::Asset,AssetLoadError> {
        builder.build(ImageState {queue: qid,
            stage: rendy::hal::pso::PipelineStage::FRAGMENT_SHADER,
            access: rendy::hal::image::Access::SHADER_READ,
            layout: rendy::hal::image::Layout::ShaderReadOnlyOptimal
        }, factory).map(B::wrap_texture).map_err(|_|AssetLoadError::UploadImageError)
    }
}

pub struct SpriteSheetLoaderInfo {
    path:String,
    config:ImageTextureConfig
}

impl SpriteSheetLoaderInfo {
    pub fn new(path: &str, config: ImageTextureConfig) -> Self { Self { path:String::from(path), config } }
    pub fn new_only_path(path: &str) -> Self { Self { path:String::from(path), config:ImageTextureConfig::default() } }
}


impl Asset for SpriteSheet {
    type LoaderInfo = SpriteSheetLoaderInfo;
}
pub struct SpriteSheetCData {
    tex_path:String,
    pub width: u32,
    pub height: u32,
    pub texture: TextureBuilder<'static>,
    sprites: Vec<Sprite>,
    name_dic: FnvHashMap<String, u32>,
    borders: FnvHashMap<String, Vec<(f32, f32, f32, f32)>>,
}

impl SpriteSheetCData {
    pub fn get_meta_data(val:&serde_json::Value) -> Option<(String,u32,u32)> {
        let meta_object = val.as_object().and_then(|o| o.get("meta"))?;
        let texture_name = meta_object.get("texture").and_then(|s| s.as_str())?;
        let w = meta_object.get("width").and_then(|s| s.as_i64())?;
        let h = meta_object.get("height").and_then(|s| s.as_i64())?;
        Some((String::from(texture_name),w as u32,h as u32))
    }

    pub fn parse_sprites(val:&Vec<serde_json::Value>,s_w:f32,s_h:f32) -> Option<Vec<Sprite>> {
        let mut ret_list = Vec::new();
        for item in val {
            if let Some(sprite_map) = item.as_object() {
                let x = sprite_map.get("x").and_then(|x| x.as_i64())?;
                let y = sprite_map.get("y").and_then(|y| y.as_i64())?;
                let width = sprite_map.get("width").and_then(|w| w.as_i64())?;
                let height = sprite_map.get("height").and_then(|h| h.as_i64())?;
                let name = sprite_map.get("name").and_then(|h| h.as_str()).map(String::from)?;
                ret_list.push(Sprite {
                    name, 
                    rect : Rect {
                        x: x as i32,y:y as i32,width:width as i32,height:height as i32
                    },
                    coord:TextureCoordinate {
                        left: x as f32 / s_w,  right:(x + width) as f32 / s_w,
                        top : y as f32 / s_h,  bottom: (y + height) as f32 / s_h
                    }
                });
            }
        }
        Some(ret_list)
    }

    pub fn parse_border(val:&Vec<serde_json::Value>) -> FnvHashMap<String,Vec<(f32,f32,f32,f32)>> {
        let mut ret_map = FnvHashMap::default();
        for item in val {
            if let Some(_border_map) = item.as_object() {
               if let Some((sprite_name,border_list)) = SpriteSheetCData::parse_border_item(&item) {
                   ret_map.insert(sprite_name.clone(),border_list);
               }
            }
        }
        ret_map
    }

    fn parse_border_item(val:&serde_json::Value) -> Option<(String,Vec<(f32,f32,f32,f32)>)> {
        let _target = val.get("target")?.as_str()?;
        let border_arr = val.get("border")?.as_array()?;
        let mut border_list:Vec<(f32,f32,f32,f32)> = Vec::new();
        for border in border_arr.iter() {
            let num_arr = border.as_array()?;
            let _0 = num_arr[0].as_f64()? as f32;
            let _1 = num_arr[1].as_f64()? as f32;
            let _2 = num_arr[2].as_f64()? as f32;
            let _3 = num_arr[3].as_f64()? as f32;
            border_list.push((_0,_1,_2,_3));
        }
        Some((String::from(_target),border_list))
    }
}

impl IAssetLoaderInfo for SpriteSheetLoaderInfo {
    type CData = SpriteSheetCData;
    type Asset = SpriteSheet;
    fn path(&self) -> &String {
        &self.path
    }
    fn load_data(&self, center:&StorageCenter, source:&LoaderEnv) -> Result<Result<Self::CData,Self::Asset>,AssetLoadError> {
        let json_bytes = source.load_fs_source(self.path.as_str())?;
        let sheet_json = serde_json::from_slice(&json_bytes).map_err(|_|AssetLoadError::FormatError)?;
        let (mut texture_path,width,height) = SpriteSheetCData::get_meta_data(&sheet_json).ok_or(AssetLoadError::FormatError)?;
        let sprite_vec = sheet_json.get("sprites").and_then(|s| s.as_array()).ok_or(AssetLoadError::FormatError)?;
        let sprites = SpriteSheetCData::parse_sprites(sprite_vec,width as f32,height as f32).ok_or(AssetLoadError::FormatError)?;
        let mut borders:FnvHashMap<String,Vec<(f32,f32,f32,f32)>> = FnvHashMap::default();
        if let Some(border_arr) = sheet_json.get("sliceBorder").and_then(|s| s.as_array()) {
           borders = SpriteSheetCData::parse_border(border_arr);
        }
        let mut name_dic = FnvHashMap::default();
        let mut i = 0;
        for sprite in sprites.iter() {
            name_dic.insert(sprite.name.clone(),i);
            i+= 1;
        }
        if let Some(parent) = Path::new(self.path.as_str()).parent() {
            let jpath = parent.join(&texture_path);
            texture_path = String::from(jpath.to_owned().to_str().unwrap());
        }
        if let Some((_,asset_id)) = center.get_asset_id(&texture_path) {
            let sprite_sheet = SpriteSheet::new(width,height,Handle::new(asset_id),sprites,name_dic,borders);
            return  Ok(Err(sprite_sheet));
        }

        let tex_load_info = TextuteLoaderInfo::new(texture_path.as_str(), self.config.clone());
        let tex_builder = tex_load_info.load_data(center, source)?.unwrap();
        Ok(Ok(SpriteSheetCData {
            tex_path:texture_path,
            width,
            height,
            texture:tex_builder,
            sprites,
            name_dic,
            borders
        }))
    }


    fn load<B:Backend>(cdata:Self::CData, factory:&mut Factory<B>, qid:QueueId,center:&StorageCenter,world:&World) -> Result<Self::Asset,AssetLoadError> {
        let texture = TextuteLoaderInfo::load(cdata.texture, factory, qid,center,world)?;
        let hid = center.insert_asset::<Texture>(texture,&cdata.tex_path,world);
        let sheet = SpriteSheet::new(cdata.width,cdata.height,hid,cdata.sprites,cdata.name_dic,cdata.borders);
        Ok(sheet)
    }
}

pub struct FontAssetLoaderInfo {
    path:String
}

impl FontAssetLoaderInfo {
    pub fn new(path: &str) -> Self { Self { path :String::from(path) } }
}


impl Asset for FontAsset {
    type LoaderInfo = FontAssetLoaderInfo;
}

impl IAssetLoaderInfo for FontAssetLoaderInfo {
    type CData = ();
    type Asset = FontAsset;

    fn path(&self) -> &String {
        &self.path
    }

    fn load_data(&self, _:&StorageCenter, source:&LoaderEnv) -> Result<Result<Self::CData,Self::Asset>,AssetLoadError> {
        let bytes = source.load_fs_source(self.path.as_str())?;
        let font_asset = Font::from_bytes(bytes).map(|font| FontAsset {font} ).map_err(|_| AssetLoadError::FormatError)?;
        Ok(Err(font_asset))
    }

   
}