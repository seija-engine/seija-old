use crate::render::types::{Backend};
use crate::render::utils::uniform::{DynamicUniform};
use rendy::factory::{Factory};
use crate::render::pod::{ViewArgs};
use specs::{World};
use crate::render::CameraGatherer;
use rendy::command::{RenderPassEncoder};
use rendy::hal::{device::{OutOfMemory}};
#[derive(Debug)]
pub struct CameraEnv<B:Backend> {
    uniform: DynamicUniform<B, ViewArgs>
}

impl<B: Backend> CameraEnv<B> {
    pub fn new(factory: &Factory<B>) -> Result<Self, OutOfMemory> {
        Ok(Self {
            uniform: DynamicUniform::new(factory, rendy::hal::pso::ShaderStageFlags::VERTEX)?,
        })
    }

    pub fn raw_layout(&self) -> &B::DescriptorSetLayout {
        self.uniform.raw_layout()
    }

    pub fn process(&mut self, factory: &Factory<B>, index: usize, world: &World) {
        let projview = CameraGatherer::gather(world).projview;
        self.uniform.write(factory, index, projview);
    }

    #[inline]
    pub fn bind(&self,index: usize,pipeline_layout: &B::PipelineLayout,set_id: u32,encoder: &mut RenderPassEncoder<'_, B>) {
        self.uniform.bind(index, pipeline_layout, set_id, encoder);
    }
}