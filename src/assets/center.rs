use crate::assets::{Asset,Handle,AssetStorage};
use std::any::{TypeId};
use std::sync::{Arc,RwLock};
use std::collections::{HashMap};
use specs::{World};


pub type AssetID = (TypeId,u32);
pub struct StorageCenter {
    assets:Arc<RwLock<HashMap<String,AssetID>>>,
}

impl Default for StorageCenter {
    fn default() -> Self {
        Self {assets:Arc::new(RwLock::new(HashMap::new())) }
    }
}

impl StorageCenter {
    pub fn contains(&self,path:&String) -> bool {
        let read = self.assets.read().unwrap();
        read.contains_key(path)
    }

    pub fn get_asset_id(&self,path:&String) -> Option<AssetID> {
        let read = self.assets.read().unwrap();
        read.get(path).map(|id| id.clone())
    }

    pub fn insert_asset<A:Asset>(&self,asset:A,path:&String,world:&World) -> Handle<A> {
       if let Some((_,asset_id)) = self.get_asset_id(path) {
           return  Handle::new(asset_id);
       }
       let hid = world.fetch_mut::<AssetStorage<A>>().insert(asset);
       let mut write = self.assets.write().unwrap();
       write.insert(path.clone(), (TypeId::of::<A>(),hid.id()));
       hid
    }
}