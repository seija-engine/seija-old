use glsl_layout::AsStd140;
use core::marker::PhantomData;
use rendy::hal::{pso::{Descriptor,ShaderStageFlags,DescriptorType},device::{OutOfMemory}};
use rendy::factory::{Factory};
use rendy::hal::buffer;
use rendy::memory::{MappedRange,Write};
use crate::render::types::{Backend};
use rendy::hal::{device::Device as _};
use rendy::resource::{Buffer,BufferInfo,Escape,Handle,DescriptorSet,DescriptorSetLayout};
use crate::render::utils::{set_layout_bindings,desc_write,slice_as_bytes};
use rendy::command::{RenderPassEncoder};


#[derive(Debug)]
pub struct DynamicUniform<B: Backend, T: AsStd140>  where T::Std140: Sized {
    layout: Handle<DescriptorSetLayout<B>>,
    per_image: Vec<PerImageDynamicUniform<B, T>>
}

#[derive(Debug)]
pub struct PerImageDynamicUniform <B: Backend, T: AsStd140> where T::Std140: Sized {
    buffer: Escape<Buffer<B>>,
    set: Escape<DescriptorSet<B>>,
    marker: PhantomData<T>,
}

impl<B: Backend, T: AsStd140> DynamicUniform<B, T> where T::Std140: Sized {
    pub fn new(factory:&Factory<B>,flags:ShaderStageFlags) -> Result<Self, OutOfMemory> {
        let layout_bind = set_layout_bindings(Some((1,DescriptorType::UniformBuffer,flags)));
        let desc_set_layout = factory.create_descriptor_set_layout(layout_bind)?;
        Ok(DynamicUniform {
            layout : desc_set_layout.into(),
            per_image : Vec::new()
        })
    }

    pub fn raw_layout(&self) -> &B::DescriptorSetLayout {
        self.layout.raw()
    }

    pub fn write(&mut self, factory: &Factory<B>, index: usize, item: T::Std140) -> bool {
        let mut changed = false;
        let this_image = {
            while self.per_image.len() <= index {
                self.per_image
                    .push(PerImageDynamicUniform::new(factory, &self.layout));
                changed = true;
            }
            &mut self.per_image[index]
        };

        let mut mapped = this_image.map(factory);
        let mut writer = unsafe {
            mapped
                .write::<u8>(factory.device(), 0..std::mem::size_of::<T::Std140>() as u64)
                .unwrap()
        };
        let slice = unsafe { writer.slice() };

        slice.copy_from_slice(slice_as_bytes(&[item]));
        changed
    }

    #[inline]
    pub fn bind(&self,index: usize,pipeline_layout: &B::PipelineLayout,binding_id: u32,encoder: &mut RenderPassEncoder<'_, B>) {
        self.per_image[index].bind(pipeline_layout, binding_id, encoder);
    }
}


impl<B: Backend, T: AsStd140> PerImageDynamicUniform<B, T> where T::Std140: Sized {
    fn new(factory: &Factory<B>, layout: &Handle<DescriptorSetLayout<B>>) -> Self {
        let buffer = factory.create_buffer(BufferInfo {
                           size: std::mem::size_of::<T::Std140>() as u64,
                           usage: buffer::Usage::UNIFORM}, rendy::memory::Dynamic).unwrap();
        let set = factory.create_descriptor_set(layout.clone()).unwrap();
        let desc = Descriptor::Buffer(buffer.raw(), None..None);
        unsafe {
            let set = set.raw();
            factory.write_descriptor_sets(Some(desc_write(set, 0, desc)));
        }
        Self {
            buffer,
            set,
            marker: PhantomData,
        }
    }

    fn map<'a>(&'a mut self, factory: &Factory<B>) -> MappedRange<'a, B> {
        let range = 0..self.buffer.size();
        self.buffer.map(factory.device(), range).unwrap()
    }

    #[inline]
    fn bind(&self,pipeline_layout: &B::PipelineLayout,set_id: u32,encoder: &mut RenderPassEncoder<'_, B>) {
        unsafe {
            encoder.bind_graphics_descriptor_sets(
                pipeline_layout,
                set_id,
                Some(self.set.raw()),
                std::iter::empty(),
            );
        }
    }
}