use fnv::{FnvHashMap};
use std::collections::hash_map::{Entry};
use std::ops::{Range};
use derivative::Derivative;


pub trait GroupIterator<K, V>
where
    Self: Iterator<Item = (K, V)> + Sized,
    K: PartialEq,
{
    fn for_each_group<F>(self, on_group: F) where F: FnMut(K, &mut Vec<V>);
}

impl<K, V, I> GroupIterator<K, V> for I where K: PartialEq,I: Iterator<Item = (K, V)> {
    fn for_each_group<F>(self, mut on_group: F)
    where
        F: FnMut(K, &mut Vec<V>),
    {
        let mut block: Option<(K, Vec<V>)> = None;

        for (next_group_id, value) in self {
            match &mut block {
                slot @ None => {
                    let mut group_buffer = Vec::with_capacity(64);
                    group_buffer.push(value);
                    slot.replace((next_group_id, group_buffer));
                }
                Some((group_id, group_buffer)) if group_id == &next_group_id => {
                    group_buffer.push(value);
                }
                Some((group_id, ref mut group_buffer)) => {
                    let submitted_group_id = std::mem::replace(group_id, next_group_id);
                    on_group(submitted_group_id, group_buffer);
                    group_buffer.clear();
                    group_buffer.push(value);
                }
            }
        }

        if let Some((group_id, mut group_buffer)) = block.take() {
            on_group(group_id, &mut group_buffer);
        }
    }
}

#[derive(Derivative, Debug)]
#[derivative(Default(bound = ""))]
pub struct OnLevelBatch<PK,D> where PK:Eq + std::hash::Hash {
    map: fnv::FnvHashMap<PK, Vec<D>>,
    data_count: usize,
}

impl<PK,D> OnLevelBatch<PK,D> where PK:Eq + std::hash::Hash {
    pub fn clear_inner(&mut self) {
        self.data_count = 0;
        for (_, data) in self.map.iter_mut() {
            data.clear();
        }
    }

    pub fn prune(&mut self) {
        self.map.retain(|_, b| !b.is_empty());
    }

    pub fn data(&self) -> impl Iterator<Item = &Vec<D>> {
        self.map.values()
    }

    pub fn insert(&mut self,pk:PK,data:impl IntoIterator<Item = D>) {
        let instance_data = data.into_iter();
        match self.map.entry(pk) {
            Entry::Occupied(mut e) => {
                let vec = e.get_mut();
                let old_len = vec.len();
                vec.extend(instance_data);
                self.data_count += vec.len() - old_len;
            },
            Entry::Vacant(e) => {
                let collected = instance_data.collect::<Vec<_>>();
                self.data_count += collected.len();
                e.insert(collected);
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PK, Range<u32>)> {
        let mut offset = 0;
        self.map.iter().map(move |(pk, data)| {
            let range = offset..offset + data.len() as u32;
            offset = range.end;
            (pk, range)
        })
    }

    pub fn get_map(&self) -> &FnvHashMap<PK, Vec<D>> {
        &self.map
    }

    pub fn count(&self) -> usize {
        self.data_count
    }
}

#[derive(Derivative, Debug)]
#[derivative(Default(bound = ""))]
pub struct OrderedOneLevelBatch<PK,D> where PK: PartialEq {
    old_keys: Vec<(PK,u32)>,
    keys_list: Vec<(PK,u32)>,
    data_list:Vec<D>
}

impl <PK,D> OrderedOneLevelBatch<PK,D> where PK: PartialEq {
    pub fn swap_clear(&mut self) {
        std::mem::swap(&mut self.old_keys, &mut self.keys_list);
        self.keys_list.clear();
        self.data_list.clear();
    }

    pub fn insert(&mut self,pk:PK,data: impl IntoIterator<Item = D>) {
        let start = self.data_list.len() as u32;
        self.data_list.extend(data);
        let added_len = self.data_list.len() as u32 - start;
        if added_len == 0 {
            return;
        }
        match self.keys_list.last_mut() {
            Some((last_pk, last_len)) if last_pk == &pk => {
                *last_len += added_len;
            },
            _ => {
                self.keys_list.push((pk, added_len));
            }
        }
    }

    pub fn data(&self) -> &Vec<D> {
        &self.data_list
    }

    pub fn iter(&self) -> impl Iterator<Item = (&PK,Range<u32>)> {
        let mut offset = 0;
        self.keys_list.iter().map(move |(pk,size)| {
            let range = offset..offset + *size;
            offset = range.end;
            (pk,range)
        })
    }

    //pub fn iter_elem(&self) -> impl Iterator<Item = D> {
       
    //}

    pub fn changed(&self) -> bool {
        self.keys_list != self.old_keys
    }

    pub fn count(&self) -> usize {
        self.data_list.len()
    }
}


#[test]
fn test_ordered_one_level_batch() {
    let mut batch = OrderedOneLevelBatch::<u32, u32>::default();
    batch.insert(0, Some(0));
    batch.insert(0,Some(0));
    batch.insert(1,Some(0));
    

    dbg!(batch);
}