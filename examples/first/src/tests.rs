use seija::specs::{World,WorldExt,world::Builder,Entity};
use seija::render::{FontAsset};
use seija::assets::{Loader,Handle,AssetStorage,AssetPack,FontAssetLoaderInfo,TextuteLoaderInfo,SpriteSheetLoaderInfo};
use seija::s2d::{S2DLoader};
use seija::render::types;
use seija::rendy;
use seija::render::components::{Mesh2D,TextRender,SpriteRender,ImageRender,LineMode,SpriteSheet,ImageType,ImageFilledType};
use seija::common::{Transform,transform::{Parent},Rect2D,Hidden,HiddenPropagate};
use seija::render::{Transparent};
use seija::math::{Vector3,UnitQuaternion};
use seija::s2d::layout::{ScreenScaler,BaseLayout,HorizontalAlign,VerticalAlign,StackPanel,Orientation};
use seija::event::cb_event::{CABEventRoot};
use seija::event::{EventNode,GameEventType};

type DefaultBackend = rendy::vulkan::Backend;

pub trait IGameTest {
    fn start(&mut self,world:&mut World);
    fn update(&mut self,world:&mut World);
    fn quit(&mut self,world:&mut World) {}
}

fn create_orgin(world:&mut World,tex:Handle<types::Texture>) -> Entity {
    let mut render = ImageRender::new(Some(tex));
    let trans = Transform::default();
    //trans.set_position(Vector3::new(0f32,0,0));
    //trans.set_rotation_euler(0.0,0.0,45f32 * 0.0174533);
    //render.set_type(ImageType::Filled(ImageFilledType::HorizontalLeft,0.5f32));
    render.set_color(1.0f32,0.0f32,0.0f32,1.0f32);
    world.create_entity().with(render).with(trans).with(Rect2D::new(10f32,10f32,[0.5f32,0.5f32])).build()
}

fn create_image(world:&mut World,tex:Handle<types::Texture>,w:f32,h:f32,x:f32,y:f32,z:f32,t:i32,p:Option<Entity>) -> Entity {
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
    let rect = Rect2D::new(w,h,[0.7f32,0.5f32]);
    match p {
        Some(e) => {
            world.create_entity().with(render).with(Parent {entity:e}).with(Mesh2D::default())
                                 .with(rect).with(trans).build()
        },
        None => {
            world.create_entity().with(Mesh2D::default()).with(render).with(rect).with(trans).build()
        }
    }
}

fn register_event<F>(world:&mut World,is_capture:bool,e:Entity,f:F,et:GameEventType) where F:Fn(Entity,&World) + 'static + Send + Sync {
    let mut ev_storage = world.write_storage::<EventNode>();
    let mut may_ev_node = ev_storage.get_mut(e);
    if may_ev_node.is_some() {
        may_ev_node.unwrap().register(is_capture,et,f);
    } else {
        let mut ev_node = EventNode::default();
        ev_node.register(is_capture,et,f);
        ev_storage.insert(e,ev_node).unwrap();
    }
}

fn create_sprite(world:&mut World,sheet:Handle<SpriteSheet>,sprite_name:&str,x:f32,y:f32,z:f32,p:Option<Entity>) -> Entity {
    let mut sprite_render = SpriteRender::new(Some(sheet),Some(sprite_name));
    //sprite_render.set_native_size(&world.fetch::<AssetStorage<SpriteSheet>>());
    //sprite_render.set_type(ImageType::Filled(ImageFilledType::HorizontalLeft,0.6f32));
    //sprite_render.set_type(ImageType::Sliced(30f32,30f32,10f32,25f32));
    let sprite = sprite_render.get_sprite(&world.fetch::<AssetStorage<SpriteSheet>>()).unwrap();
    let mut trans = Transform::default();
    trans.set_position(Vector3::new(x,y,z));
    //trans.set_rotation_euler(0.0,0.0,45f32 * 0.0174533);
    let rect2d = Rect2D::new(sprite.rect.width as f32 * 0.5f32,sprite.rect.height as f32 * 0.5f32,[0.5f32,0.5f32]);
    match p {
        Some(e) => {
            world.create_entity().with(Mesh2D::default()).with(Parent {entity:e})
            .with(sprite_render).with(trans).with(Transparent).with(rect2d).build()
                         
        },
        None => {
            world.create_entity().with(Mesh2D::default()).with(sprite_render).with(trans).with(Transparent).with(rect2d).build()
        }
    }
}

fn create_sprite2(world:&mut World,sheet:Handle<SpriteSheet>,sprite_name:&str,w:f32,h:f32) -> Entity {
    let mut sprite_render = SpriteRender::new(Some(sheet),Some(sprite_name));
    sprite_render.set_slice_type_by_cfg(0,&world.fetch::<AssetStorage<SpriteSheet>>());
    let trans = Transform::default();
    let rect2d = Rect2D::new(w,h,[0.5f32,0.5f32]);
    world.create_entity().with(Mesh2D::default()).with(sprite_render).with(trans).with(Transparent).with(rect2d).build()
}

fn create_text(world:&mut World,font:Handle<FontAsset>,text:&str,x:f32,y:f32,z:f32,p:Option<Entity>) -> Entity {
    let mut trans = Transform::default();
    //trans.set_rotation_euler(0.0,0.0,45f32 * 0.0174533);
    let mut render = TextRender::new(Some(font));
    render.set_text(text);
    render.set_font_size(16);
    render.set_color(1f32,1f32,1f32,1f32);
    render.set_line_mode(LineMode::Wrap);
    trans.set_position(Vector3::new(x,y,z));
    match p {
        Some(e) => {
            world.create_entity().with(Parent {entity:e}).with(Mesh2D::default())
            .with(Transparent).with(render).with(Rect2D::new(200f32,100f32,[0.5f32,0.5f32])).with(trans).build()
        },
        None => {
            world.create_entity().with(Transparent).with(Mesh2D::default())
            .with(render).with(Rect2D::new(200f32,100f32,[0.5f32,0.5f32])).with(trans).build()
        }
    }
}
/**********************************FontTest**********************************************/
#[derive(Default)]
pub struct FontTest {}

impl IGameTest for FontTest {
    fn start(&mut self,world:&mut World) {
        let font =  {
            let loader = world.write_resource::<S2DLoader>();
            loader.load_sync::<_,DefaultBackend>(FontAssetLoaderInfo::new("WenQuanYiMicroHei.ttf") ,world).unwrap()
        };
        //for i in 1..200 {
            let mut trans = Transform::default();
            //trans.set_position(Vector3::new(-220f32,0f32,0f32));
            trans.set_rotation_euler(0.0,0.0,45f32 * 0.0174533);
            let mut render = TextRender::new(Some(font.clone()) );
            render.set_text("确定");
            render.set_font_size(16);
            render.set_line_mode(LineMode::Wrap);
            render.set_color(0.0f32,0.0f32,0.0f32,1.0f32);
            world.create_entity().with(Transparent).with(render).with(trans).with(Rect2D::new(200f32,200f32,[0.5f32,0.5f32])).build();   
        //}   
    }

    fn update(&mut self,world:&mut World) {}
}
/**********************************SpriteTest**********************************************/
#[derive(Default)]
pub struct SpriteTest {}

impl IGameTest for SpriteTest {
    fn start(&mut self,world:&mut World) {
        let (sprite_sheet,font) = {
            let loader = world.write_resource::<S2DLoader>();
            ( 
              //loader.load_sync::<_,DefaultBackend>(TextuteLoaderInfo::new_only_path("a.jpg"),world).unwrap(),
              //loader.load_sync::<_,DefaultBackend>(TextuteLoaderInfo::new_only_path("Button.png"),world).unwrap(),
              loader.load_sync::<SpriteSheetLoaderInfo,DefaultBackend>(SpriteSheetLoaderInfo::new_only_path("111/material.json"),world).unwrap(),
              loader.load_sync::<_,DefaultBackend>(FontAssetLoaderInfo::new("WenQuanYiMicroHei.ttf"),world).unwrap()
            )
        };
        
        //create_image(world,tex_btn.clone(),111f32 * 4f32,51f32 *4f32,0f32,0f32,1f32,2,None);
        //create_image(world,tex_btn.clone(),111f32 ,51f32 ,0f32,0f32,1f32,0,None);
        //create_image(world,tex_a.clone(),100f32,100f32,0f32,0f32,2f32,0,None);
        //create_orgin(world,tex_a.clone());
        //for _ in 0..2000 {
        //create_image(world,tex_a.clone(),100f32,100f32,100f32,100f32,1.1f32,None);
        //}
        let btn = create_sprite2(world,sprite_sheet,"button-active",100f32,30f32);
        create_text(world,font,"Click Me!",0f32,0f32,0f32,Some(btn));
        register_event(world,false,btn,|e,_w| {
            println!("Move:{}",e.id());
        },GameEventType::Click);
    }

    fn update(&mut self,world:&mut World) {
    }
}

/*************************************EventTest******************************************* */
#[derive(Default)]
pub struct EventTest {

}

impl IGameTest for EventTest {
    fn start(&mut self,world:&mut World) {
        let ui_root = world.create_entity().with(Transform::default())
                                           .with(Rect2D::default())
                                           .with(ScreenScaler::with_scale_height(768f32))
                                           .with(CABEventRoot::default()).build();
        let tex_a = {
            let loader = world.write_resource::<S2DLoader>();
            loader.load_sync::<_,DefaultBackend>(TextuteLoaderInfo::new_only_path("a.jpg"),world).unwrap()
        };
        let e = create_image(world,tex_a.clone(),100f32,100f32,0f32,0f32,2f32,0,Some(ui_root));
        let e2 = create_image(world,tex_a.clone(),50f32,50f32,0f32,0f32,0f32,0, Some(e));
        //let e3 = create_image(world,tex_a.clone(),30f32,30f32,0f32,0f32,0f32,0, Some(e2));
       
        register_event(world,false,e,|e,_w| {
            println!("MouseEnter:{}",e.id());
        },GameEventType::MouseEnter);

        register_event(world,false,e,|e,_w| {
            println!("MouseLeave:{}",e.id());
        },GameEventType::MouseLeave);
    }

    fn update(&mut self,world:&mut World) {

    }
}
/*************************************UITest******************************************* */
#[derive(Default)]
pub struct UITest { }

impl IGameTest for UITest  {
    fn start(&mut self,world:&mut World) {
        let (sprite_sheet,font) = {
            let loader = world.write_resource::<S2DLoader>();
            ( 
              loader.load_sync::<_,DefaultBackend>(SpriteSheetLoaderInfo::new_only_path("111/paper.json"),world).unwrap(),
              loader.load_sync::<_,DefaultBackend>(FontAssetLoaderInfo::new("WenQuanYiMicroHei.ttf"),world).unwrap()
            )
        };
        let ui_root = world.create_entity().with(Transform::default())
                                           .with(Rect2D::new(1024f32,768f32,[0.5f32,0.5f32]))
                                           .with(CABEventRoot::default()).build();

        let ebtn = create_sprite(world,sprite_sheet.clone(),"BlueButton",0f32,-200f32,1f32,Some(ui_root));
        let epress = create_sprite(world,sprite_sheet.clone(),"BlueButtonPressed",0f32,0f32,0f32,Some(ebtn));
        {
            let mut hidden_storage = world.write_storage::<Hidden>();
            hidden_storage.insert(epress,Hidden).unwrap();
        };
        let text = create_text(world,font,"关闭",0f32,0f32,0f32,Some(ebtn));
        register_event(world,true,ebtn,move |e,world| {
            let mut hidden_storage = world.write_storage::<Hidden>();
            hidden_storage.remove(epress);
        },GameEventType::TouchStart);

        let bg = create_sprite(world,sprite_sheet.clone(),"SmallBackground",0f32,50f32,1f32,Some(ui_root));
        let pic = create_sprite(world,sprite_sheet.clone(),"StarIcon",0f32,0f32,0f32,Some(bg));

       
        register_event(world,true,ebtn,move |e,world| {
            let mut hidden_storage = world.write_storage::<Hidden>();
            hidden_storage.insert(epress,Hidden).unwrap();
        },GameEventType::TouchEnd);

        
        register_event(world,true,ebtn,move |e,world| {
            let mut hidden_storage = world.write_storage::<HiddenPropagate>();
            if  hidden_storage.contains(bg) {
                hidden_storage.remove(bg).unwrap();
                let mut text_storage = world.write_storage::<TextRender>();
                let text_render = text_storage.get_mut(text).unwrap();
                text_render.set_text("关闭");
            } else {
                let mut text_storage = world.write_storage::<TextRender>();
                let text_render = text_storage.get_mut(text).unwrap();
                text_render.set_text("打开");
                hidden_storage.insert(bg,HiddenPropagate).unwrap();
            }
        },GameEventType::Click);
    }

    fn update(&mut self,world:&mut World) {

    }
}

/*************************************LayoutTest******************************************* */
#[derive(Default)]
pub struct LayoutTest { }

impl IGameTest for LayoutTest {
    fn start(&mut self, world:&mut World) {
        let a_tex = {
            let loader = world.write_resource::<S2DLoader>();
            loader.load_sync::<_,DefaultBackend>(TextuteLoaderInfo::new_only_path("./a.jpg"),world).unwrap()
        }; 

        let ui_root = world.create_entity().with(Transform::default())
                                                   .with(Rect2D::default())
                                                   .with(ScreenScaler::with_scale_height(768f32))
                                                   .with(CABEventRoot::default()).build();
        {
            let mut stack = StackPanel::default();
            stack.spacing = 10f32;
            stack.orientation = Orientation::Horizontal;
            let mut stack_storage = world.write_storage::<StackPanel>();
            
            stack_storage.insert(ui_root, stack).unwrap();
        }
       /* {
            let elImage = create_image(world,a_tex.clone(),0f32,10f32,0f32,0f32,1f32,0,Some(ui_root));
            let mut layout_storage = world.write_storage::<BaseLayout>();
            let mut layout = BaseLayout::default();
            layout.horizontal_align = HorizontalAlign::Stretch;
            layout.vertical_align = VerticalAlign::Bottom;
            layout.margin.set_all(10f32);
            layout_storage.insert(elImage,layout).unwrap();
        } */
        for idx in 0..5 {
            let cimg = { create_image(world,a_tex.clone(),200f32,100f32,0f32,0f32,1f32,0,Some(ui_root)) };
            
        }
        
        
    }

    fn update(&mut self, _world:&mut World) {
        
    }
}