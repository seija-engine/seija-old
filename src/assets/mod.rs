mod impls;
mod center;
mod pack;
mod env;
mod errors;
mod storage;
mod loader;
pub use env::{Source,LocalFS,LoaderEnv};
pub use errors::{AssetLoadError};
pub use center::{StorageCenter,AssetID};
pub use storage::{AssetStorage,Handle};
pub use pack::{S2DAssetPack};
pub use loader::{Loader};

use crate::render::types::{Backend};
use rendy::factory::{Factory};
use rendy::command::{QueueId};
use specs::{World};
pub use impls::{TextuteLoaderInfo,SpriteSheetLoaderInfo,FontAssetLoaderInfo};


pub trait IAssetLoaderInfo {
    type CData;
    type Asset:Asset;
    fn path(&self) -> &String;
    fn load_data(&self,center:&center::StorageCenter,source:&LoaderEnv) -> Result<Result<Self::CData,Self::Asset>,AssetLoadError>;
    fn load<B:Backend>(_:Self::CData,_:&mut Factory<B>,_:QueueId,_:&center::StorageCenter,_:&World) -> Result<Self::Asset,AssetLoadError> {
        unimplemented!()
    }
}

pub trait Asset : Send + Sync + 'static {
    type LoaderInfo:IAssetLoaderInfo;
}

pub trait AssetPack : Send + Sync + 'static {

}