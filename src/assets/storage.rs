use crate::assets::{Asset};
use specs::storage::{VecStorage,UnprotectedStorage};
use hibitset::BitSet;
use std::marker::PhantomData;
use std::sync::Arc;
use derivative::Derivative;
use std::sync::atomic::{AtomicUsize, Ordering};
use crossbeam_queue::SegQueue;

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Debug(bound = "")
)]
pub struct Handle <A :?Sized> {
    id: Arc<u32>,
    #[derivative(Debug = "ignore")]
    marker: PhantomData<A>,
}

impl<A> Handle<A> {
    pub fn id(&self) -> u32 {
        *self.id.as_ref()
    }

    /*
    fn is_unique(&self) -> bool {
        Arc::strong_count(&self.id) == 1
    }*/

    pub fn new(id:u32) -> Self {
        Handle {
            id: Arc::new(id),
            marker: PhantomData
        }
    }
}


#[derive(Debug, Default)]
pub struct Allocator {
    store_count: AtomicUsize,
}

impl Allocator {
    pub fn next_id(&self) -> usize {
        self.store_count.fetch_add(1, Ordering::Relaxed)
    }
}



pub struct AssetStorage<A:Asset> {
    assets: VecStorage<(A, u32)>,
    bitset: BitSet,
    handles: Vec<Handle<A>>,
    handle_alloc: Allocator,
    unused_handles: SegQueue<Handle<A>>
}

impl<A: Asset> AssetStorage<A> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn allocate_new(&self) -> Handle<A> {
        let id = self.handle_alloc.next_id() as u32;
        Handle {
            id:Arc::new(id),
            marker: PhantomData
        }
    }

    pub fn allocate(&self) -> Handle<A> {
        self.unused_handles.pop().unwrap_or_else(|_| self.allocate_new())
    }

    pub fn clone_asset(&mut self,_handle:&Handle<A>) -> Option<Handle<A>> where A:Clone {
        None
    }

    pub fn get(&self,handle:&Handle<A>) -> Option<&A> {
        if self.bitset.contains(handle.id()) {
            Some(unsafe { &self.assets.get(handle.id()).0 })
        } else {
            None
        }
    }

    pub fn get_by_id(&self,id:u32) -> Option<&A> {
        if self.bitset.contains(id) {
            Some(unsafe { &self.assets.get(id).0 })
        } else {
            None
        }
    }

    pub unsafe fn get_by_id_unchecked(&self,id:u32) -> &A {
        &self.assets.get(id).0
    }

    pub fn get_mut(&mut self,handle:&Handle<A>) -> Option<&mut A> {
        if self.bitset.contains(handle.id()) {
            Some(unsafe { &mut self.assets.get_mut(handle.id()).0 })
        } else {
            None
        }
    }

    pub fn replace(&mut self,handle:&Handle<A>,asset:A) -> A {
        if self.bitset.contains(handle.id()) {
            let data = unsafe { self.assets.get_mut(handle.id()) };
            data.1 += 1;
            std::mem::replace(&mut data.0, asset)
        } else {
            panic!("Trying to replace not loaded asset");
        }
    }

    pub fn insert(&mut self,asset:A) -> Handle<A> {
        let handle = self.allocate();
        let id = handle.id();
        self.bitset.add(id);
        self.handles.push(handle.clone());
        unsafe {
            self.assets.insert(id, (asset,0))
        }
        handle
    }

    pub fn contains(&self,handle:&Handle<A>) -> bool {
        self.bitset.contains(handle.id())
    }

    pub fn contains_id(&self,id:u32) -> bool {
        self.bitset.contains(id)
    }

    pub fn get_version(&self, handle: &Handle<A>) -> Option<u32> {
        if self.bitset.contains(handle.id()) {
            Some(unsafe { self.assets.get(handle.id()).1 })
        } else {
            None
        }
    }

    pub fn get_with_version(&self, handle: &Handle<A>) -> Option<&(A, u32)> {
        if self.bitset.contains(handle.id()) {
            Some(unsafe { self.assets.get(handle.id()) })
        } else {
            None
        }
    }

    pub fn unload_all(&mut self) {
        unsafe {self.assets.clean(&self.bitset) }
        self.bitset.clear();
    }
}

impl<A: Asset> Default for AssetStorage<A> {
    fn default() -> Self {
        AssetStorage {
            assets: Default::default(),
            bitset: Default::default(),
            handles: Default::default(),
            handle_alloc: Default::default(),
            unused_handles: Default::default()
        }
    }
}