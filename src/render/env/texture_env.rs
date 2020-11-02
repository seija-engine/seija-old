use crate::render::types::{Backend,Texture};
use rendy::factory::Factory;
use rendy::command::{RenderPassEncoder};
use rendy::resource::{Handle as RendyHandle,DescriptorSetLayout,Escape,DescriptorSet};
use rendy::hal::pso::{ShaderStageFlags,CreationError};
use  rendy::hal::{device::Device,image};
use crate::assets::{AssetStorage,Handle};
use specs::{World};
use crate::render::utils::{LookupBuilder,desc_write,texture_desc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(u32);

#[derive(Debug)]
pub struct TextureSet<B:Backend> {
    set:Escape<DescriptorSet<B>>,
    handle:Handle<Texture>,
    layout: image::Layout
}

#[derive(Debug)]
pub struct TextureEnv<B:Backend> {
    layout: RendyHandle<DescriptorSetLayout<B>>,
    textures:Vec<TextureSet<B>>,
    lookup:LookupBuilder<u32>
}

impl<B: Backend> TextureEnv<B> {
    pub fn new(factory: &Factory<B>) -> Result<Self, CreationError> {
        Ok(Self {
            layout: set_layout! {factory, [1] CombinedImageSampler ShaderStageFlags::FRAGMENT},
            textures: Vec::with_capacity(1024),
            lookup: LookupBuilder::new()
        })
    }

    pub fn raw_layout(&self) -> &B::DescriptorSetLayout {
        self.layout.raw()
    }
    

    pub fn insert(&mut self,factory: &Factory<B>,world:&World,handle:&Handle<Texture>, layout: image::Layout) -> Option<TextureId> {
        let id = self.lookup.forward(handle.id());
        if self.textures.get(id).is_some() {
            return Some(TextureId(id as u32));
        }
       
        let tex_storage = world.fetch::<AssetStorage<Texture>>();
        let tex = tex_storage.get(handle)?;
        let set = factory.create_descriptor_set(self.layout.clone()).unwrap();
        unsafe {
            let desc = texture_desc(tex, layout)?;
            factory.write_descriptor_sets(vec![desc_write(set.raw(), 0, desc)]);
            
        };
        let tex_set = TextureSet {
            set,
            handle:handle.clone(),
            layout
        };
        if self.textures.len() == id {
            self.textures.push(tex_set);
        } else {
            self.textures[id] = tex_set;
        };
        Some(TextureId(id as u32))
    }

    pub fn bind(&self,pipeline_layout: &B::PipelineLayout,set_id: u32,texture_id: TextureId,encoder: &mut RenderPassEncoder<'_, B>) {
        let tex_set = &self.textures[texture_id.0 as usize];
        unsafe {
            encoder.bind_graphics_descriptor_sets(pipeline_layout,set_id,Some(tex_set.set.raw()),std::iter::empty());
        }
    }
}