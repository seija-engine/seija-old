use crate::assets::{AssetPack,AssetStorage};
use specs::{World};
use crate::render::types::{Texture};
use crate::render::components::{SpriteSheet};
use crate::render::{FontAsset};
pub enum S2DAssetPack {

}

impl AssetPack for S2DAssetPack {
    
}


impl S2DAssetPack {
    pub fn register_all_storage(world:&mut World) {
        world.insert(AssetStorage::<Texture>::new());
        world.insert(AssetStorage::<SpriteSheet>::new());
        world.insert(AssetStorage::<FontAsset>::new());
    }
}