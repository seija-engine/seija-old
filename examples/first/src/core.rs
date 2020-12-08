use seija::specs::{World,WorldExt,world::Builder,Entity};
use seija::assets::{Handle};
use seija::render::types;
use seija::render::components::{Mesh2D,ImageRender,ImageType,ImageFilledType};
use seija::common::{Transform,Rect2D};

use seija::math::{Vector3};

pub fn create_image(world:&mut World,tex:Handle<types::Texture>,w:f32,h:f32,x:f32,y:f32,z:f32,t:i32) -> Entity {
    let mut render = ImageRender::new(Some(tex));
    let mut trans = Transform::default();
    trans.set_position(Vector3::new(x,y,z));
    //trans.set_rotation_euler(0.0,0.0,45f32 * 0.0174533);
    if t == 1 {
        render.set_type(ImageType::Filled(ImageFilledType::HorizontalLeft,0.6f32));
    } else if t == 2 {
        render.set_type(ImageType::Sliced(30f32,30f32,10f32,25f32));
        //render.set_anchor(0f32,0f32);
    }
    let rect = Rect2D::new(w,h,[0.5f32,0.5f32]);
    world.create_entity().with(Mesh2D::default()).with(render).with(rect).with(trans).build()
}