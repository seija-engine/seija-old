use crate::assets::{IAssetLoaderInfo,StorageCenter,AssetPack,LoaderEnv,Handle,AssetLoadError};
use specs::{World};
use crate::render::types::{Backend};
use std::marker::PhantomData;
use rendy::factory::{Factory};
use rendy::command::{QueueId};
pub struct Loader<T:AssetPack> {
    env:LoaderEnv,
    m:PhantomData<T>
}

impl<T> Default for Loader<T> where T:AssetPack {
    fn default() -> Self {
        Loader {
            env:LoaderEnv::default(), m: PhantomData
        }
    }
}

impl<T> Loader<T> where T:AssetPack {
    pub fn env(&self) -> &LoaderEnv {
        &self.env
    }
    pub fn load_sync<AL:IAssetLoaderInfo,B:Backend>(&self,info:AL,world:&World) -> Result<Handle<AL::Asset>,AssetLoadError>  {
        let center_ref = world.fetch::<StorageCenter>();
        let ret = info.load_data(&center_ref, &self.env)?;
        match ret {
            Ok(cdata) => {
                let qid = *world.fetch::<QueueId>();
                let load_asset = AL::load(cdata,&mut world.fetch_mut::<Factory<B>>(),qid,&center_ref,world)?;
                let hid = center_ref.insert_asset::<AL::Asset>(load_asset,info.path(),world);
                Ok(hid)
            },
            Err(asset) => {
               let hid = center_ref.insert_asset::<AL::Asset>(asset,info.path(),world);
               Ok(hid)
            }
        } 
    }
}