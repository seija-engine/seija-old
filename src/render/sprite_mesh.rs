use crate::render::pod::{SpriteArg,Vertex2D};
use crate::assets::{AssetStorage};
use crate::render::types::{Texture,Backend};
use crate::render::components::{ImageRender,Mesh2D,SpriteRender,TextRender,SpriteSheet};
use specs::{System,Read,ReadStorage,WriteStorage,Join,Write};
use crate::common::{Transform,Rect2D};
use rendy::factory::{Factory};
use rendy::command::{QueueId,RenderPassEncoder};
use std::marker::PhantomData;
use crate::render::{FontAsset,env::{FontEnv}};
use crate::render::utils::vertex::{DynamicVertexBuffer,DynamicIndexBuffer};

pub type SpriteMeshId = usize;

#[derive(Debug,Clone)]
pub struct SpriteMesh {
    pub sprite_arg:SpriteArg,
    pub meshes:Vec<Vertex2D>,
    pub indexs:Vec<u16>
}


pub struct SpriteMeshSystem<B:Backend> {
    marker: PhantomData<B>
}

impl<B> SpriteMeshSystem<B> where B:Backend {
    pub fn new() -> Self {
       SpriteMeshSystem {
           marker:PhantomData
       }
    }
}

impl<'a,B:Backend> System<'a> for SpriteMeshSystem<B> {
    
    type SystemData = (
        Write<'a,AssetStorage<Texture>>,
        Read<'a,AssetStorage<SpriteSheet>>,
        Write<'a,FontEnv<B>>,
        WriteStorage<'a,ImageRender>,
        WriteStorage<'a,SpriteRender>,
        WriteStorage<'a,TextRender>,
        WriteStorage<'a,Mesh2D>,
        ReadStorage<'a,Transform>,
        ReadStorage<'a,Rect2D>,
        Option<Read<'a,QueueId>>,
        Option<Write<'a,Factory<B>>>,
        Read<'a, AssetStorage<FontAsset>>,
    );

    fn run(&mut self,(mut tex_storage,sprite_sheet_storage,
                      mut font_env,mut images,mut sprites,
                      mut texts,mut mesh2ds,trans,rects,may_qid,may_factory,font_storage):Self::SystemData) {

        for (img,mesh2d,t,rect) in (&mut images,&mut mesh2ds,&trans,&rects).join() {
            if let Some(tex_id) = img.texture.as_ref() {
                let tex = tex_storage.get(tex_id).unwrap();
                img.process_mesh(&t,tex.texture_size(),&rect,mesh2d);
            }
        }

        for (sprite,mesh2d,t,rect) in (&mut sprites,&mut mesh2ds,&trans,&rects).join() {
            sprite.process_mesh(&t,&sprite_sheet_storage,&rect,mesh2d);
        }

        let text_iter:(&mut  WriteStorage<'a,TextRender>,
                       &mut WriteStorage<'a,Mesh2D>,
                       &ReadStorage<'a,Transform>,&ReadStorage<'a,Rect2D>) = (&mut texts,&mut mesh2ds,&trans,&rects);
        let qid = may_qid.unwrap();
        let mut factory = may_factory.unwrap();
        font_env.process(&mut tex_storage,&font_storage,text_iter,*qid,&mut factory);
    }
}



#[derive(Debug,Default)]
struct MeshElementInfo {
    pub index_start:u32,
    pub index_end:u32,
    pub vert_start:u32,
    pub vert_end:u32,
    pub instance:u32,
}
#[derive(Debug)]
pub struct SpriteDynamicMesh<B:Backend> {
    mesh_info:Vec<MeshElementInfo>,
    vertexs:Vec<Vertex2D>,
    indexs:Vec<u16>,
    instances:Vec<SpriteArg>,
    vertex_buffer: DynamicVertexBuffer<B, Vertex2D>,
    vertex_index_buffer: DynamicIndexBuffer<B, u16>,
    vertex_instance_buffer:DynamicVertexBuffer<B,SpriteArg>,
}

impl<B> Default for SpriteDynamicMesh<B> where B:Backend {
    fn default() -> Self {
        SpriteDynamicMesh {
            mesh_info:vec![],
            vertexs:vec![],
            indexs:vec![],
            instances:vec![],
            vertex_buffer: DynamicVertexBuffer::new(),
            vertex_index_buffer: DynamicIndexBuffer::new(),
            vertex_instance_buffer:  DynamicVertexBuffer::new(),
        }
    }
}

impl<B> SpriteDynamicMesh<B> where B:Backend {
    pub fn insert(&mut self,mesh:&SpriteMesh) -> usize {
        let mut info = MeshElementInfo::default();
        let self_index_len = self.indexs.len() as u32;
        let self_vert_len = self.vertexs.len() as u16;
        info.index_start = self_index_len;
        info.index_end = (self.indexs.len() + mesh.indexs.len()) as u32;
        info.vert_start = self.vertexs.len() as u32;
        info.vert_end = (self.vertexs.len() + mesh.meshes.len()) as u32;
        info.instance = self.mesh_info.len() as u32;

        self.mesh_info.push(info);
        self.vertexs.extend(&mesh.meshes);
        self.indexs.extend(mesh.indexs.iter().map(move |idx| idx + self_vert_len));
        self.instances.push(mesh.sprite_arg);

        self.mesh_info.len() - 1        
    }

    pub fn clear(&mut self) {
        self.vertexs.clear();
        self.indexs.clear();
        self.instances.clear();
        self.mesh_info.clear();
    }

    pub fn bind(&self,index: usize, encoder: &mut RenderPassEncoder<'_, B>) {
        self.vertex_instance_buffer.bind(index, 1, 0,encoder);
        self.vertex_index_buffer.bind(index, 0,encoder);
        self.vertex_buffer.bind(index, 0,0, encoder);
    }

    pub fn write(&mut self,factory:&Factory<B>,index:usize) {
        self.vertex_buffer.write_list(factory, index, self.vertexs.len() as u64, &self.vertexs);
        self.vertex_index_buffer.write_list(factory,index,self.indexs.len() as u64,&self.indexs);
        self.vertex_instance_buffer.write_list(factory,index,self.instances.len() as u64,&self.instances);
    }

    pub fn draw_index(&mut self,idx:&usize, encoder: &mut RenderPassEncoder<'_, B>) {
       let mesh_info = unsafe { self.mesh_info.get_unchecked(*idx) };
       unsafe {
         encoder.draw_indexed(mesh_info.index_start..mesh_info.index_end, 0,mesh_info.instance..mesh_info.instance + 1)
       };
       //dbg!(&mesh_info);
    }
}