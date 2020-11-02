use rendy::hal:: image::{Kind};

pub trait Backend:rendy::hal::Backend {
    
    fn unwrap_texture(texture: &Texture) -> Option<&rendy::texture::Texture<Self>>;
    
    fn wrap_texture(texture: rendy::texture::Texture<Self>) -> Texture;
}

#[derive(Debug)]
pub enum Texture {
    Vulkan(rendy::texture::Texture<rendy::vulkan::Backend>),
}




impl Texture {
    pub fn texture_size(&self) -> (u32,u32) {
        match self {
            Texture::Vulkan(tex) => {
                let kind = tex.image().kind();
                if let Kind::D2(w,h,_,_) = kind {
                    (w,h)
                } else {
                    (0,0)
                }
            }
        }
    }
}

impl Backend for rendy::vulkan::Backend {
    fn unwrap_texture(texture: &Texture) -> Option<&rendy::texture::Texture<Self>> {
        match texture {
            Texture::Vulkan(ref inner) => Some(inner)
        }
    }
    
    fn wrap_texture(texture: rendy::texture::Texture<Self>) -> Texture {
        Texture::Vulkan(texture)
    }
}