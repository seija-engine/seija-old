use std::path::{PathBuf};
use std::fs::{File};
use crate::assets::{AssetLoadError};
use std::collections::HashMap;
use std::sync::{RwLock};
pub trait Source: Send + Sync + 'static {
    fn load(&self, path: &str) -> Result<Vec<u8>, AssetLoadError>;
    fn set_loc(&mut self,path:&str);
    fn loc(&self) -> String;
}

#[derive(Debug)]
pub struct LocalFS {
    loc: PathBuf
}

impl LocalFS {
    pub fn new<P>(loc: P) -> Self where P: Into<PathBuf> {
        LocalFS { loc: loc.into() }
    }

    fn path(&self, s_path: &str) -> PathBuf {
        let mut path = self.loc.clone();
        path.push(s_path);
        path
    }
}

impl Source for LocalFS {
    fn load(&self, path: &str) -> Result<Vec<u8>, AssetLoadError> {
        use std::io::Read;
        let path = self.path(path);
        let mut v = Vec::new();
        let mut file = File::open(&path).map_err(|_| AssetLoadError::LoadFileError)?;
        file.read_to_end(&mut v).map_err(|_| AssetLoadError::LoadFileError)?;
        Ok(v)
    }

    fn set_loc(&mut self,path: &str) {
        self.loc = PathBuf::new();
        self.loc.push(path);
    }

    fn loc(&self) -> String {
        String::from(self.loc.to_str().unwrap_or_default()) 
    }
}



pub struct LoaderEnv {
    sources:HashMap<String,RwLock<Box<dyn Source>>>
}

impl Default for LoaderEnv {
    fn default() -> LoaderEnv {
        let mut env = LoaderEnv {
            sources: HashMap::default()
        };
        env.sources.insert(String::from("fs"),RwLock::new(Box::new(LocalFS::new(""))));
        env
    }
}

impl LoaderEnv {
    pub fn set_source_root(&self,source_type:&str,path:&str) -> bool {
        if let Some(box_source) = self.sources.get(&String::from(source_type)) {
            box_source.write().unwrap().set_loc(path);
            return true;
        }
        false
    }

    pub fn get_source_root(&self,source_type:&str) -> Option<String> {
        if let Some(box_source) = self.sources.get(&String::from(source_type)) {
            return Some(box_source.read().unwrap().loc());
        }
        None
    }

    
    pub fn set_fs_root(&self,path:&str) -> bool {
        self.set_source_root("fs", path)
    }

    pub fn fs_root(&self) -> String {
        self.get_source_root("fs").unwrap_or_default()
    }

    pub fn load_by_source(&self,source_type:&str,path:&str) -> Result<Vec<u8>, AssetLoadError> {
        let box_source = self.sources.get(&String::from(source_type)).ok_or(AssetLoadError::LoadFileError)?;
        box_source.read().unwrap().load(path)
    }

    pub fn load_fs_source(&self,path:&str) -> Result<Vec<u8>, AssetLoadError> {
        self.load_by_source("fs", path)
    }
}