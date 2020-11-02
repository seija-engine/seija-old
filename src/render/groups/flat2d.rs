use crate::render::pipeline::{PipelineDescBuilder, PipelinesBuilder};
use crate::render::pod::{Vertex2D,SpriteArg};
use crate::assets::{AssetStorage};
use crate::render::utils::{simple_shader_set,ChangeDetection};
use crate::common::{Transform};
use rendy::command::{QueueId, RenderPassEncoder};
use rendy::factory::{Factory};
use specs::{Read,ReadStorage,World,SystemData,Join,Entities};
use crate::render::types::{Backend};
use crate::render::{SpriteVisibility,SpriteMeshId,SpriteDynamicMesh};
use crate::render::components::{ImageRender,SpriteRender,SpriteSheet,TextRender,Mesh2D};
use crate::render::batch::{OnLevelBatch,OrderedOneLevelBatch};
use rendy::graph::{
    render::{PrepareResult, RenderGroup, RenderGroupDesc},
    GraphContext, NodeBuffer, NodeImage,
};
use rendy::hal::{
    self,
    device::Device as _,
    pso::{Primitive, ShaderStageFlags},
};
use rendy::hal::{pass::Subpass, pso, pso::{CreationError}};
use rendy::mesh::AsVertex;
use rendy::shader::{Shader, SpirvShader};
use crate::render::env::{CameraEnv,TextureEnv,TextureId,FontEnv};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

lazy_static::lazy_static! {
    static ref SPRITE_VERTEX:SpirvShader = SpirvShader::from_bytes(
      include_bytes!("../shaders/compiled/sprite.vert.spv"),
      ShaderStageFlags::VERTEX,
      "main").unwrap();
    static ref SPRITE_FRAGMENT:SpirvShader = SpirvShader::from_bytes(
      include_bytes!("../shaders/compiled/sprite.frag.spv"),
      ShaderStageFlags::VERTEX,
      "main").unwrap();
}

#[derive(Debug)]
pub struct Flat2DGroupDesc;

impl Flat2DGroupDesc {
    pub fn new() -> Self {
        Flat2DGroupDesc
    }
}

impl<B: Backend> RenderGroupDesc<B, World> for Flat2DGroupDesc {
    fn build<'a>(self,_ctx: &GraphContext<B>,factory: &mut Factory<B>,_queue: QueueId,_world: &World,
                 framebuffer_width: u32,framebuffer_height: u32,subpass: Subpass<'_, B>,_buffers: Vec<NodeBuffer>,_images: Vec<NodeImage>) 
                 -> Result<Box<dyn RenderGroup<B, World>>, CreationError> {
        
        let camera_env = CameraEnv::new(factory)?;
        let texture_env = TextureEnv::new(factory)?;
        
        let (pipeline, pipeline_layout) = build_sprite_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            false,vec![camera_env.raw_layout(),texture_env.raw_layout()]).unwrap();
        Ok(Box::new(Flat2DGroup {
            camera_env,
            texture_env,
            pipeline,
            pipeline_layout,
            sprites:Default::default(),
            dynamic_mesh:SpriteDynamicMesh::default()
        }))
    }

}


#[derive(Debug)]
pub struct Flat2DGroup<B: Backend> {
    camera_env: CameraEnv<B>,
    pipeline: B::GraphicsPipeline,
    pipeline_layout:B::PipelineLayout,
    sprites:OnLevelBatch<TextureId,SpriteMeshId>,
    texture_env: TextureEnv<B>,
    dynamic_mesh:SpriteDynamicMesh<B>
}

impl<B: Backend> RenderGroup<B, World> for Flat2DGroup<B> {
    fn prepare(&mut self,factory: &Factory<B>,_queue: QueueId,index: usize,_subpass: Subpass<'_, B>,aux: &World) -> PrepareResult {
        #[cfg(feature = "profiler")]
        profile_scope!("Flat2DGroup prepare");
        let (entities,image_renders,transforms,visibility,mesh2ds) = <(
            Entities<'_>,
            ReadStorage<'_, ImageRender>,
            ReadStorage<'_, Transform>,
            Read<'_, SpriteVisibility>,
            ReadStorage<'_,Mesh2D>,
        )>::fetch(aux);

        self.camera_env.process(factory,index,aux);
        let sprites_ref = &mut self.sprites;
        let textures_ref = &mut self.texture_env;
        sprites_ref.clear_inner();
        self.dynamic_mesh.clear();
        
        for (_,img,_,_,mesh2d) in (&entities,&image_renders,&transforms,&visibility.visible_unordered,&mesh2ds).join() {
            let mesh = mesh2d.mesh.as_ref();
            if mesh.is_some() && img.texture.is_some() {
                let tex_id = textures_ref.insert(factory,aux,img.texture.as_ref().unwrap(),hal::image::Layout::ShaderReadOnlyOptimal).unwrap();
                let did = self.dynamic_mesh.insert(mesh.unwrap());
                sprites_ref.insert(tex_id, Some(did));
            }
        };
       
        self.dynamic_mesh.write(factory,index);
        sprites_ref.prune();
        PrepareResult::DrawRecord
    }

    fn draw_inline(&mut self,mut encoder: RenderPassEncoder<'_, B>,index: usize,_subpass: Subpass<'_, B>,_aux: &World) {
        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.camera_env.bind(index, layout, 0, &mut encoder);
        self.dynamic_mesh.bind(index,&mut encoder);
        for (&tex, lst) in self.sprites.get_map().iter() {
            self.texture_env.bind(layout, 1, tex, &mut encoder);
            for mesh_id in lst {
                self.dynamic_mesh.draw_index(mesh_id,&mut encoder);
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &World) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory.device().destroy_pipeline_layout(self.pipeline_layout);
        };
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct Flat2DTransparentDesc;

impl Flat2DTransparentDesc {
    pub fn new() -> Self {
        Flat2DTransparentDesc
    }
}

impl<B: Backend> RenderGroupDesc<B, World> for Flat2DTransparentDesc {
    fn build<'a>(self,_ctx: &GraphContext<B>,factory: &mut Factory<B>,_queue: QueueId,_aux: &World,
                 framebuffer_width: u32,framebuffer_height: u32,subpass: Subpass<'_, B>,_buffers: Vec<NodeBuffer>,_images: Vec<NodeImage>) 
                 -> Result<Box<dyn RenderGroup<B, World>>, CreationError> {
        let camera_env = CameraEnv::new(factory)?;
        let texture_env = TextureEnv::new(factory)?; 
       
        let (pipeline, pipeline_layout) = build_sprite_pipeline(
             factory,
             subpass,
             framebuffer_width,
             framebuffer_height,
             true,vec![camera_env.raw_layout(),texture_env.raw_layout()]).unwrap();
        Ok(Box::new(Flat2DTransparent {
            camera_env,
            texture_env,
            dynamic_mesh:SpriteDynamicMesh::default(),
            pipeline,
            pipeline_layout,
            sprites:OrderedOneLevelBatch::default(),
            change:ChangeDetection::default()
        }))
    }

}

#[derive(Debug)]
pub struct Flat2DTransparent<B: Backend> {
    camera_env: CameraEnv<B>,
    texture_env: TextureEnv<B>,
    dynamic_mesh:SpriteDynamicMesh<B>,
    pipeline: B::GraphicsPipeline,
    pipeline_layout:B::PipelineLayout,
    sprites:OrderedOneLevelBatch<TextureId,SpriteMeshId>,
    change: ChangeDetection,
}

impl<B: Backend> RenderGroup<B, World> for Flat2DTransparent<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {
        let (sprite_sheet_storage,
            img_renders,
            sprite_renders,
            mes2des,
            transforms,
            visibility,
            font_env,
            texts
        ) = <(
            Read<'_, AssetStorage<SpriteSheet>>,
            ReadStorage<'_,ImageRender>,
            ReadStorage<'_, SpriteRender>,
            ReadStorage<'_,Mesh2D>,
            ReadStorage<'_, Transform>,
            Read<'_, SpriteVisibility>,
            Read<'_,FontEnv<B>>,
            ReadStorage<'_,TextRender>
        )>::fetch(world);
        self.camera_env.process(factory,index,world);
        self.sprites.swap_clear();
        let textures_ref = &mut self.texture_env;
        let sprites_ref = &mut self.sprites;
        self.dynamic_mesh.clear();
        
        let mut image_joined = (&img_renders, &transforms,&mes2des).join();
        let mut sprite_joined = (&sprite_renders,&transforms,&mes2des).join();
        let mut text_joined = (&texts,&transforms,&mes2des).join();
        let font_tex_id = textures_ref.insert(factory,world,font_env.font_tex.as_ref().unwrap(),hal::image::Layout::ShaderReadOnlyOptimal).unwrap();
        
        for e in visibility.visible_ordered.iter() {
           let may_image = image_joined.get_unchecked(e.id());
           if let Some((image,_,mesh2d)) = may_image {
                if let Some(ref mesh) = mesh2d.mesh {
                    if let Some(tex_id) = image.texture.as_ref() {
                        let tex_id = textures_ref.insert(factory,world,tex_id,hal::image::Layout::ShaderReadOnlyOptimal).unwrap();
                        let did = self.dynamic_mesh.insert(mesh);
                        sprites_ref.insert(tex_id, Some(did));
                    }
                }
           }
           let may_sprite = sprite_joined.get_unchecked(e.id());
           if let Some((sprite,_,mesh2d)) = may_sprite {
            if let Some(ref mesh) = mesh2d.mesh  {
                if let Some(sheet) = sprite.sprite_sheet.as_ref() {
                    let ref_tex = &sprite_sheet_storage.get(sheet).unwrap().texture;
                    let tex_id = textures_ref.insert(factory,world,ref_tex,hal::image::Layout::ShaderReadOnlyOptimal).unwrap();
                    let did = self.dynamic_mesh.insert(mesh);
                    sprites_ref.insert(tex_id, Some(did));
                }
            }
           }

           let may_text = text_joined.get_unchecked(e.id());
           if let Some((_,_,mesh2d)) = may_text {
                if let Some(ref mesh) = mesh2d.mesh  {
                    let did = self.dynamic_mesh.insert(mesh);
                    sprites_ref.insert(font_tex_id, Some(did));
                }
           }
        }

        self.dynamic_mesh.write(factory,index);
        PrepareResult::DrawRecord
    }

    fn draw_inline(&mut self,mut encoder: RenderPassEncoder<'_, B>,index: usize,_subpass: hal::pass::Subpass<'_, B>,_world: &World) {
        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.camera_env.bind(index, layout, 0, &mut encoder);

        self.dynamic_mesh.bind(index,&mut encoder);
        let data_list = self.sprites.data();
        for (&tex,range) in self.sprites.iter() {
            self.texture_env.bind(layout, 1, tex, &mut encoder);
            for idx in range {
                let mesh_id = unsafe { data_list.get_unchecked(idx as usize) };
                self.dynamic_mesh.draw_index(mesh_id,&mut encoder);
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &World) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory.device().destroy_pipeline_layout(self.pipeline_layout);
        }
    }

}

fn build_sprite_pipeline<B: Backend>(factory: &Factory<B>,subpass: Subpass<'_, B>,framebuffer_width: u32,
                                     framebuffer_height: u32,transparent: bool,layouts: Vec<&B::DescriptorSetLayout>) 
                                     -> Result<(B::GraphicsPipeline, B::PipelineLayout), failure::Error> {
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
            .unwrap()
    };
    let shader_vertex = unsafe { SPRITE_VERTEX.module(factory).unwrap() };
    let shader_fragment = unsafe { SPRITE_FRAGMENT.module(factory).unwrap() };

    let vert_format = vec![(Vertex2D::vertex(), pso::VertexInputRate::Vertex),(SpriteArg::vertex(),pso::VertexInputRate::Instance(1))];
    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&vert_format)
                .with_input_assembler(pso::InputAssemblerDesc::new(Primitive::TriangleList))
                .with_shaders(simple_shader_set(&shader_vertex, Some(&shader_fragment)))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_blend_targets(vec![pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: if transparent {
                        Some(pso::BlendState::PREMULTIPLIED_ALPHA)
                    } else {
                        None
                    },
                }])
                .with_depth_test(pso::DepthTest {
                    fun: pso::Comparison::Less,
                    write: !transparent,
                }),
        )
        .build(factory, None);
    unsafe {
        factory.destroy_shader_module(shader_vertex);
        factory.destroy_shader_module(shader_fragment);
    }

    match pipes {
        Err(e) => {
            unsafe {
                factory.device().destroy_pipeline_layout(pipeline_layout);
            }
            Err(e)
        }
        Ok(mut pipes) => Ok((pipes.remove(0), pipeline_layout)),
    }
}
