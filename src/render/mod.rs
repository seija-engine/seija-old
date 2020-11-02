mod render;
mod graph_node;
mod render_plan; 
mod camera;
mod gather;
mod font;
pub mod batch;
pub mod types;
pub mod components;
mod sprite_mesh;
#[macro_use]
pub mod macros;
pub mod pod;
pub mod pipeline;
pub mod utils;
pub mod groups;
pub mod env;
mod sprite_visibility;
mod transparent;
pub use transparent::{Transparent};
pub use sprite_visibility::{SpriteVisibilitySortingSystem,SpriteVisibility};
use rendy::hal;
use rendy::wsi::Surface;
use rendy::hal::{Backend};
pub use render::{RenderSystem,RenderBuilder};
pub use graph_node::{GraphNodeBuilder,GraphNode};
pub use camera::{Camera,ActiveCamera};
pub use gather::{CameraGatherer};
pub use font::{FontAsset};
pub use sprite_mesh::{SpriteMeshSystem,SpriteMesh,SpriteMeshId,SpriteDynamicMesh};

#[derive(Debug, Copy,Clone)]
pub struct ImageOptions {
    pub kind: hal::image::Kind,
    pub levels: hal::image::Level,
    pub format: hal::format::Format,
    pub clear: Option<hal::command::ClearValue>,
}

#[derive(Debug)]
pub enum OutputColor<B: Backend> {
    Image(ImageOptions),
    Surface(Surface<B>, Option<hal::command::ClearValue>),
}

#[derive(Debug)]
pub struct OutputOptions<B:Backend> {
    pub colors: Vec<OutputColor<B>>,
    pub depth: Option<ImageOptions>,
}

#[derive(Debug)]
#[repr(i32)]
pub enum RenderOrder {
    BeforeOpaque = 90,
    Opaque = 100,
    AfterOpaque = 110,

    BeforeTransparent = 190,
    Transparent = 200,
    AfterTransparent = 210,

    LinearPostEffects = 300,
    ToneMap = 400,
    DisplayPostEffects = 500,
    Overlay = 600,
}

impl Into<i32> for RenderOrder {
    fn into(self) -> i32 {
        self as i32
    }
}