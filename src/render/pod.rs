use glsl_layout::*;
use rendy::{
    hal::format::Format,
    mesh::{AsVertex, VertexFormat},
};




#[derive(Clone, Copy, Debug, AsStd140)]
#[repr(C, align(16))]
pub struct ViewArgs {
    /// Projection matrix
    pub proj: mat4,
    /// View matrix
    pub view: mat4,
    /// Premultiplied Proj-View matrix
    pub proj_view: mat4,
}

#[derive(Clone, Copy, Debug, AsStd140, PartialEq, PartialOrd)]
#[repr(C, align(4))]
pub struct SpriteArg {
    pub model: mat4,
    pub color: vec4
}

impl AsVertex for SpriteArg {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Format::Rgba32Sfloat, "model"),
            (Format::Rgba32Sfloat, "model"),
            (Format::Rgba32Sfloat, "model"),
            (Format::Rgba32Sfloat, "model"),
            (Format::Rgba32Sfloat,"color")
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(4))]
pub struct Vertex2D {
    pub pos:vec3,
    pub uv:vec2
}

impl AsVertex for Vertex2D {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Format::Rgb32Sfloat,"pos"),
            (Format::Rg32Sfloat,"uv")
        ))
    }
}


pub trait IntoPod<T> {
    /// Converts `Self` to the supplied `T` GLSL type.
    fn into_pod(self) -> T;
}

impl IntoPod<vec3> for nalgebra::Vector3<f32> {
    fn into_pod(self) -> vec3 {
        let arr: [f32; 3] = self.into();
        arr.into()
    }
}

impl IntoPod<vec2> for nalgebra::Vector2<f32> {
    fn into_pod(self) -> vec2 {
        let arr: [f32; 2] = self.into();
        arr.into()
    }
}

impl IntoPod<vec4> for nalgebra::Vector4<f32> {
    fn into_pod(self) -> vec4 {
        let arr: [f32; 4] = self.into();
        arr.into()
    }
}