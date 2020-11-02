use crate::render::types::{Backend};
use rendy::resource::{Buffer,Escape};
use rendy::factory::{Factory};
use rendy::hal;
use rendy::command::{RenderPassEncoder};
use std::ops::Range;
use rendy::memory::{MappedRange,Write};
use crate::render::utils;
use std::marker::PhantomData;

pub trait VertexDataBufferType {
    fn usage() -> hal::buffer::Usage; 
}

pub type DynamicVertexBuffer<B, T> = DynamicVertexData<B, VertexData<B, T>, T>;

pub type DynamicIndexBuffer<B, T> = DynamicVertexData<B, IndexData<B, T>, T>;

#[derive(Debug)]
pub struct IndexData<B, T>(PhantomData<(B, T)>);

impl<B: Backend, T: 'static> VertexDataBufferType for IndexData<B, T> {
    #[inline]
    fn usage() -> hal::buffer::Usage {
        hal::buffer::Usage::INDEX
    }
}

impl<B: Backend> IndexData<B, u16> {
    #[inline]
    pub fn bind(
        encoder: &mut RenderPassEncoder<'_, B>,
        buffer: &Option<Escape<Buffer<B>>>,
        offset: u64,
    ) -> bool {
        if let Some(buffer) = buffer.as_ref() {
            unsafe {
                encoder.bind_index_buffer(buffer.raw(), offset, hal::IndexType::U16);
            }
            return true;
        }

        false
    }
}

impl<B: Backend> IndexData<B, u32> {
    #[inline]
    pub fn bind(
        _: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
        buffer: &Option<Escape<Buffer<B>>>,
        offset: u64,
    ) -> bool {
        if let Some(buffer) = buffer.as_ref() {
            unsafe {
                encoder.bind_index_buffer(buffer.raw(), offset, hal::IndexType::U32);
            }
            return true;
        }

        false
    }
}

#[derive(Debug)]
pub struct VertexData<B, T>(PhantomData<(B, T)>);

impl<B: Backend, T: 'static> VertexDataBufferType for VertexData<B, T> {
    #[inline]
    fn usage() -> hal::buffer::Usage {
        hal::buffer::Usage::VERTEX
    }
}

impl<B: Backend, T: 'static> VertexData<B, T> {
    #[inline]
    pub fn bind(
        binding_id: u32,
        encoder: &mut RenderPassEncoder<'_, B>,
        buffer: &Option<Escape<Buffer<B>>>,
        offset: u64,
    ) -> bool {
        if let Some(buffer) = buffer.as_ref() {
            unsafe {
                encoder.bind_vertex_buffers(binding_id, Some((buffer.raw(), offset)));
            }
            return true;
        }

        false
    }
}

#[derive(Debug)]
pub struct DynamicVertexData<B:Backend,V:VertexDataBufferType,T:'static> {
    per_image: Vec<PerImageDynamicVertexData<B, V>>,
    marker:PhantomData<T>
}

impl<B:Backend,V:VertexDataBufferType,T:'static> DynamicVertexData<B,V,T> {
    pub fn new() -> Self {
        Self {
            per_image: Vec::new(),
            marker: PhantomData,
        }
    }

    pub fn write<I>(&mut self,factory:&Factory<B>,index:usize,max_num_items:u64,iter:I) -> bool 
        where I:IntoIterator,I::Item:AsRef<[T]>{
            if max_num_items == 0 {
                return false;
            }
            let this_image = {
                while self.per_image.len() <= index {
                    self.per_image.push(PerImageDynamicVertexData::new());
                }
                &mut self.per_image[index]
            };
            let buf_size = max_num_items * std::mem::size_of::<T>() as u64;
            if let Some((allocated, mut mapped)) = this_image.map(factory, 0..buf_size) {
                let mut writer = unsafe { mapped.write::<u8>(factory.device(), 0..buf_size).unwrap() };
                let mut slice = unsafe { writer.slice() };
                iter.into_iter().for_each(|data| {
                    let data_slice = utils::slice_as_bytes(data.as_ref());
                    let tmp = std::mem::replace(&mut slice, &mut []);
                    let (dst_slice, rest) = tmp.split_at_mut(data_slice.len());
                    dst_slice.copy_from_slice(data_slice);
                    let _ = std::mem::replace(&mut slice, rest);
                });
                allocated
            } else {
                false
            }
    }

    pub fn write_list(&mut self,factory:&Factory<B>,index:usize,max_item:u64,data_list:&[T]) -> bool  {
        if max_item == 0 {
            return false;
        }
        let this_image = {
            while self.per_image.len() <= index {
                self.per_image.push(PerImageDynamicVertexData::new());
            }
            &mut self.per_image[index]
        };
        let buf_size = max_item * std::mem::size_of::<T>() as u64;
        if let Some((allocated, mut mapped)) = this_image.map(factory, 0..buf_size) {
            let mut writer = unsafe { mapped.write::<u8>(factory.device(), 0..buf_size).unwrap() };
            let data_slice = utils::slice_as_bytes(data_list);
            writer.write(&data_slice);
            //let mut slice = unsafe { writer.slice() };
            //let data_slice = utils::slice_as_bytes(data_list);
            //let tmp = std::mem::replace(&mut slice, &mut []);
            //let (dst_slice, rest) = tmp.split_at_mut(data_slice.len());
            //dst_slice.copy_from_slice(data_slice);
            //std::mem::replace(&mut slice, rest);
            allocated
        } else {
            false
        }
    }
}

impl<B: Backend, T: 'static> DynamicVertexData<B, VertexData<B, T>, T> {
    #[inline]
    pub fn bind(
        &self,
        index: usize,
        binding_id: u32,
        offset: u64,
        encoder: &mut RenderPassEncoder<'_, B>,
    ) -> bool {
        self.per_image.get(index).map_or(false, |i| {
            VertexData::<B, T>::bind(binding_id, encoder, &i.buffer, offset)
        })
    }
}

impl<B: Backend> DynamicVertexData<B, IndexData<B, u16>, u16> {
    #[inline]
    pub fn bind(&self, index: usize, offset: u64, encoder: &mut RenderPassEncoder<'_, B>) -> bool {
        self.per_image.get(index).map_or(false, |i| {
            IndexData::<B, u16>::bind(encoder, &i.buffer, offset)
        })
    }
}

impl<B: Backend> DynamicVertexData<B, IndexData<B, u32>, u32> {
    #[inline]
    pub fn bind(&self, index: usize, offset: u64, encoder: &mut RenderPassEncoder<'_, B>) -> bool {
        self.per_image.get(index).map_or(false, |i| {
            IndexData::<B, u16>::bind(encoder, &i.buffer, offset)
        })
    }
}

#[derive(Debug)]
struct PerImageDynamicVertexData<B:Backend,V:VertexDataBufferType> {
    buffer:Option<Escape<Buffer<B>>>,
    marker:PhantomData<V>
}

impl<B: Backend, V: VertexDataBufferType> PerImageDynamicVertexData<B, V> {
    fn new() -> Self {
        Self {
            buffer: None,
            marker: PhantomData,
        }
    }

    fn ensure(&mut self,factory:&Factory<B>,max_size:u64) -> bool {
        utils::ensure_buffer(&factory,
            &mut self.buffer,
            V::usage(),
            rendy::memory::Dynamic,
            max_size).unwrap()
    }

    fn map<'a>(&'a mut self,factory:&Factory<B>,range:Range<u64>) -> Option<(bool,MappedRange<'a,B>)> {
        let alloc = self.ensure(factory,range.end);
        if let Some(buffer) = &mut self.buffer {
            Some((alloc,buffer.map(factory.device(), range).unwrap()))
        } else {
            None
        }
    }
}