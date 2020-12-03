use crate::render::types::{Backend,Texture};
use rendy::command::{QueueId};
use crate::assets::{Handle,AssetStorage};
use crate::common::{Transform,Rect2D,Horizontal,Vertical};
use specs::{WriteStorage,ReadStorage,Join};
use rendy::factory::{Factory,ImageState};
use glyph_brush::rusttype::{Scale,Rect as FontRect};
use glyph_brush::{GlyphBrushBuilder,FontId,GlyphBrush,BrushAction,
                  SectionText,Layout,BuiltInLineBreaker,VariedSection,
                  LineBreaker,LineBreak,HorizontalAlign,VerticalAlign};
use crate::render::components::{TextRender,LineMode,Mesh2D};
use rendy::texture::{TextureBuilder,pixel::{R8Unorm},Texture as RTexture};
use crate::render::pod::{Vertex2D,SpriteArg};
use crate::render::{FontAsset,SpriteMesh};
use rendy::hal;
use std::collections::{HashMap};
use std::marker::PhantomData;

pub struct FontEnv<B:Backend> {
    pub font_tex:Option<Handle<Texture>>,
    glyph_brush:GlyphBrush<'static,Vec<Vertex2D>>,
    fonts_map: HashMap<u32, FontId>,
    mark: PhantomData<B>,
    cache_meshs:HashMap<String,SpriteMesh> //todo gc it
}

impl<B> Default for FontEnv<B> where B:Backend {
    fn default() -> Self {
        FontEnv {
            font_tex:None,
            glyph_brush:GlyphBrushBuilder::using_fonts(vec![]).initial_cache_size((512, 512)).build(),
            fonts_map: HashMap::new(),
            cache_meshs:HashMap::new(),
            mark:PhantomData
        }
    }
}


#[derive(Debug, Hash, Clone, Copy)]
enum CustomLineBreaker {
    BuiltIn(BuiltInLineBreaker),
    None,
}

impl LineBreaker for CustomLineBreaker {
    fn line_breaks<'a>(&self, glyph_info: &'a str) -> Box<dyn Iterator<Item = LineBreak> + 'a> {
        match self {
            CustomLineBreaker::BuiltIn(inner) => inner.line_breaks(glyph_info),
            CustomLineBreaker::None => Box::new(std::iter::empty()),
        }
    }
}


impl<B> FontEnv<B> where B:Backend {
    pub fn process<'a>(&mut self,tex_storage:&mut AssetStorage<Texture>,font_storage:&AssetStorage<FontAsset>,
                       text_iter:(&mut WriteStorage<'a,TextRender>,&mut WriteStorage<'a,Mesh2D>,
                                  &ReadStorage<'a,Transform>,&ReadStorage<Rect2D>),qid:QueueId,
                       factory:&mut Factory<B>) {
        if self.font_tex.is_none() {
            let (w, h) = self.glyph_brush.texture_dimensions();
            let tex = create_font_texture(w, h,qid,factory);
            let tex_id = tex_storage.insert(tex);
            self.font_tex = Some(tex_id);
        };
        let font_tex_handle = self.font_tex.as_ref().unwrap();
        let font_tex = tex_storage.get(font_tex_handle).and_then(B::unwrap_texture).expect("Glyph texture is created synchronously");
        for (text,mesh2d,t,rect) in text_iter.join() {
            if !text.is_valid() {
                continue;
            }
            if self.cache_meshs.contains_key(&text.text) {
                if mesh2d.mesh.is_none() || mesh2d.is_dirty {
                    if let Some(cache_mesh) = self.cache_meshs.get(&text.text) {
                        mesh2d.mesh = Some(cache_mesh.clone())
                    }
                }
                if let Some(mesh) = mesh2d.mesh.as_mut() {
                      let  mat:[[f32; 4]; 4] = (*t.global_matrix()).into();
                      mesh.sprite_arg.model = mat.into();
                }
                continue
            }
            let text_font = text.font.as_ref().unwrap();
            let mut font_lookup = None;
            if self.fonts_map.contains_key(&text_font.id()) {
                font_lookup = Some(*self.fonts_map.get(&text_font.id()).unwrap());
            } else {
                let may_font_asset = font_storage.get(text_font);
                if let Some(font_asset) = may_font_asset {
                  let font_id =  self.glyph_brush.add_font(font_asset.font.clone());
                  self.fonts_map.insert(text_font.id(), font_id);
                  font_lookup = Some(font_id);
                }
            };

            let font_id = font_lookup.unwrap();
            let section_text =  vec![SectionText {
                text: text.text.as_str(),
                scale: Scale::uniform(text.font_size as f32),
                color: text.color.into(),
                font_id
            }];
            let (h,v) = text.anchor.to_hv_align();
            let layout = match text.line_mode {
                LineMode::Single => Layout::SingleLine {
                    line_breaker: CustomLineBreaker::None,
                    h_align: h.into(),
                    v_align: v.into()
                },
                LineMode::Wrap => Layout::Wrap {
                    line_breaker: CustomLineBreaker::BuiltIn(
                        BuiltInLineBreaker::UnicodeLineBreaker,
                    ),
                    h_align: h.into(),
                    v_align: v.into(),
                }
            };
            
            let global = t.global_matrix();
            let col3 = global.column(3);
            let section = VariedSection {
                screen_position:(0f32,0f32),
                bounds:(rect.width,rect.height),
                z:col3[2],
                layout:Default::default(),
                text:section_text
            };
            
            self.glyph_brush.queue_custom_layout(section, &layout);
            let action = self.glyph_brush.process_queued(|rect,data| {
                Self::update_text_texture(factory,rect,data,font_tex,&qid);
            }, move |vert| {
                let z = vert.z;
                let left = vert.pixel_coords.min.x as f32;
                let right = vert.pixel_coords.max.x as f32;
                let top = -vert.pixel_coords.min.y as f32;
                let bottom = -vert.pixel_coords.max.y as f32;
                let uv = vert.tex_coords;
                vec![Vertex2D {
                      pos:[left,top,z].into(),
                      uv:[uv.min.x,uv.min.y].into()
                    },
                    Vertex2D {
                        pos:[right,top,z].into(),
                        uv:[uv.max.x,uv.min.y].into()
                    },
                    Vertex2D {
                        pos:[left,bottom,z].into(),
                        uv:[uv.min.x,uv.max.y].into()
                    },
                    Vertex2D {
                        pos:[right,bottom,z].into(),
                        uv:[uv.max.x,uv.max.y].into()
                    },
                ]
            }).unwrap();
            match action {
                BrushAction::Draw(verts_list) => {
                    let  mat:[[f32; 4]; 4] = (*t.global_matrix()).into();
                    let mut indexs:Vec<u16> = Vec::with_capacity(verts_list.len() * 6);
                    for idx in 0..verts_list.len() {
                       let index = idx as u16 * 4u16;
                       indexs.push(index + 0u16);
                       indexs.push(index + 1u16);
                       indexs.push(index + 2u16);
                       indexs.push(index + 1u16);
                       indexs.push(index + 3u16);
                       indexs.push(index + 2u16);
                    }
                    let mut meshes:Vec<Vertex2D> = Vec::with_capacity(verts_list.len() * 4);
                    for verts in verts_list {
                        meshes.extend(verts);
                    }
                    let text_mesh = SpriteMesh {
                        sprite_arg:SpriteArg {
                            model:mat.into(),
                            color:text.color.into(),
                        },
                        meshes,
                        indexs
                    };
                    mesh2d.mesh = Some(text_mesh.clone());
                    self.cache_meshs.insert(text.text.clone(),text_mesh);
                },
                BrushAction::ReDraw => ()
            };
        }
    }

    fn update_text_texture(factory:&mut Factory<B>,rect:FontRect<u32>,data:&[u8],tex:&RTexture<B>,qid:&QueueId) {
        unsafe {
            factory.upload_image(tex.image().clone(),
                rect.width(),
                rect.height(),
                hal::image::SubresourceLayers {
                    aspects: hal::format::Aspects::COLOR,
                    level: 0,
                    layers: 0..1,
                },
                hal::image::Offset {
                    x: rect.min.x as _,
                    y: rect.min.y as _,
                    z: 0,
                },
                hal::image::Extent {
                    width: rect.width(),
                    height: rect.height(),
                    depth: 1,
                },
                data,
                ImageState {
                    queue: *qid,
                    stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                    access: hal::image::Access::SHADER_READ,
                    layout: hal::image::Layout::General,
                },
                ImageState {
                    queue: *qid,
                    stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                    access: hal::image::Access::SHADER_READ,
                    layout: hal::image::Layout::General,
                }).unwrap();
        };
    }
}


fn create_font_texture<B:Backend>(w:u32,h:u32,queue:QueueId,factory:&mut Factory<B>) -> Texture {
    use hal::format::{Component as C, Swizzle};
    let image_state = ImageState {
                            queue,
                            stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                            access: hal::image::Access::SHADER_READ,
                            layout: hal::image::Layout::General
                      };
    TextureBuilder::new()
        .with_kind(hal::image::Kind::D2(w, h, 1, 1))
        .with_view_kind(hal::image::ViewKind::D2)
        .with_data_width(w)
        .with_data_height(h)
        .with_data(vec![R8Unorm { repr: [0] }; (w * h) as _])
        .with_swizzle(Swizzle(C::R, C::R, C::R, C::R))
    .build(image_state,factory).map(B::wrap_texture).expect("Failed to create glyph texture")
} 


impl Into<HorizontalAlign> for Horizontal {
    fn into(self) -> HorizontalAlign {
        match self {
            Horizontal::Left => HorizontalAlign::Left,
            Horizontal::Center => HorizontalAlign::Center,
            Horizontal::Right => HorizontalAlign::Right
        }
    }
}

impl Into<VerticalAlign> for Vertical {
    fn into(self) -> VerticalAlign { 
        match self {
            Vertical::Top => VerticalAlign::Top,
            Vertical::Center => VerticalAlign::Center,
            Vertical::Bottom => VerticalAlign::Bottom
        }
    }
}