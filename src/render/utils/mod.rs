use rendy::{hal::pso::{DescriptorSetWrite,Descriptor,DescriptorType,ShaderStageFlags,DescriptorSetLayoutBinding}};
use rendy::hal::{format,buffer::Usage};
use rendy::hal::pso;
use rendy::hal;
use rendy::memory::MemoryUsage;
use rendy::factory::{Factory};
use rendy::resource::{Buffer,Escape,BufferInfo};
use rendy::mesh::VertexFormat;
use rendy::graph::render::{PrepareResult};
use smallvec::SmallVec;
use std::hash::{Hash};
use crate::render::types::{Texture,Backend};
pub mod uniform;
pub mod vertex;
use derivative::Derivative;

pub fn set_layout_bindings(bindings: impl IntoIterator<Item = (u32,DescriptorType, ShaderStageFlags)>) -> Vec<DescriptorSetLayoutBinding> {
    bindings.into_iter()
            .flat_map(|(times, ty, stage_flags)| (0..times).map(move |_| (ty, stage_flags)))
            .enumerate()
            .map(
                |(binding, (ty, stage_flags))| DescriptorSetLayoutBinding {
                    binding: binding as u32,
                    ty,
                    count: 1,
                    stage_flags,
                    immutable_samplers: false,
                },
            ).collect()
}

#[inline]
pub fn desc_write<'a, B: Backend>(
    set: &'a B::DescriptorSet,
    binding: u32,
    descriptor: Descriptor<'a, B>,
) -> DescriptorSetWrite<'a, B, Option<Descriptor<'a, B>>> {
     DescriptorSetWrite {
        set,
        binding,
        array_offset: 0,
        descriptors: Some(descriptor),
    }
}

pub fn vertex_desc(
    formats: &[(VertexFormat, pso::VertexInputRate)],
) -> (Vec<pso::VertexBufferDesc>, Vec<pso::AttributeDesc>) {
    let mut vertex_buffers = Vec::with_capacity(formats.len());
    let mut attributes = Vec::with_capacity(formats.len());
    let mut sorted: SmallVec<[_; 16]> = formats.iter().enumerate().collect();
    sorted.sort_unstable_by(|a, b| a.1.cmp(&b.1));
    let mut loc_offset = 0;
    for (loc_base, (format, rate)) in sorted {
        push_vertex_desc(
            format.gfx_vertex_input_desc(*rate),
            loc_base as pso::Location + loc_offset,
            &mut vertex_buffers,
            &mut attributes,
        );
        loc_offset += format.attributes.len() as pso::Location - 1;
    }
    (vertex_buffers, attributes)
}

pub fn push_vertex_desc(
    (elements, stride, rate): (
        impl IntoIterator<Item = pso::Element<format::Format>>,
        pso::ElemStride,
        pso::VertexInputRate,
    ),
    mut location: pso::Location,
    vertex_buffers: &mut Vec<pso::VertexBufferDesc>,
    attributes: &mut Vec<pso::AttributeDesc>,
) {
    let index = vertex_buffers.len() as pso::BufferIndex;
    vertex_buffers.push(pso::VertexBufferDesc {
        binding: index,
        stride,
        rate,
    });
    for element in elements.into_iter() {
        attributes.push(pso::AttributeDesc {
            location,
            binding: index,
            element,
        });
        location += 1;
    }
}

pub fn simple_shader_set<'a, B: Backend>(
    vertex: &'a B::ShaderModule,
    fragment: Option<&'a B::ShaderModule>,
) -> pso::GraphicsShaderSet<'a, B> {
    simple_shader_set_ext(vertex, fragment, None, None, None)
}

pub fn simple_shader_set_ext<'a, B: Backend>(
    vertex: &'a B::ShaderModule,
    fragment: Option<&'a B::ShaderModule>,
    hull: Option<&'a B::ShaderModule>,
    domain: Option<&'a B::ShaderModule>,
    geometry: Option<&'a B::ShaderModule>,
) -> pso::GraphicsShaderSet<'a, B> {
    fn map_entry_point<B: Backend>(module: &B::ShaderModule) -> pso::EntryPoint<'_, B> {
        pso::EntryPoint {
            entry: "main",
            module,
            specialization: pso::Specialization::default(),
        }
    }

    pso::GraphicsShaderSet {
        vertex: map_entry_point(vertex),
        fragment: fragment.map(map_entry_point),
        hull: hull.map(map_entry_point),
        domain: domain.map(map_entry_point),
        geometry: geometry.map(map_entry_point),
    }
}

#[inline]
pub fn texture_desc<B: Backend>(
    texture: &Texture,
    layout: hal::image::Layout,
) -> Option<pso::Descriptor<'_, B>> {
    B::unwrap_texture(texture).map(|inner| {
        pso::Descriptor::CombinedImageSampler(inner.view().raw(), layout, inner.sampler().raw())
    })
}

pub fn ensure_buffer<B:Backend>(
    factory:&Factory<B>,
    buffer:&mut Option<Escape<Buffer<B>>>,
    usage:Usage,
    memory_usage: impl MemoryUsage,min_size: u64) -> Result<bool,failure::Error> {
        if buffer.as_ref().map(|b| b.size()).unwrap_or(0) < min_size {
            let new_size = min_size.next_power_of_two();
            let new_buffer = factory.create_buffer(BufferInfo {
                size: new_size,
                usage,
            },memory_usage)?;
            *buffer = Some(new_buffer);
            Ok(true)
        } else {
            Ok(false)
        }
}

pub fn slice_as_bytes<T>(slice: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            core::mem::size_of::<T>() * slice.len(),
        )
    }
}

#[derive(Debug, Default)]
pub struct LookupBuilder<I: Hash + Eq> {
    forward: fnv::FnvHashMap<I, usize>,
    len: usize,
}

impl<I: Hash + Eq> LookupBuilder<I> {

    pub fn new() -> LookupBuilder<I> {
        LookupBuilder {
            forward: fnv::FnvHashMap::default(),
            len: 0,
        }
    }

    pub fn forward(&mut self, id: I) -> usize {
        if let Some(&id_num) = self.forward.get(&id) {
            id_num
        } else {
            let id_num = self.len;
            self.forward.insert(id, id_num);
            self.len += 1;
            id_num
        }
    }
}


#[derive(Debug, Clone, Copy, Derivative)]
#[derivative(Default)]
pub enum ChangeDetection {   
    #[derivative(Default)]
    Stable,
    Changed(usize),
}

impl ChangeDetection {
    pub fn can_reuse(&mut self,index:usize,changed:bool) -> bool {
        match (*self,changed) {
            (_, true) => {
                *self = ChangeDetection::Changed(index);
                false
            }
            (ChangeDetection::Changed(idx), false) if idx == index => {
                *self = ChangeDetection::Stable;
                true
            }
            (ChangeDetection::Stable, false) => true,
            (ChangeDetection::Changed(_), false) => false,
        }
    }

    pub fn prepare_result(&mut self, index: usize, changed: bool) -> PrepareResult {
        if self.can_reuse(index, changed) {
            PrepareResult::DrawReuse
        } else {
            PrepareResult::DrawRecord
        }
    }
}