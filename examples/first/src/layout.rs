use seija::{assets::Handle, render::types::Texture, s2d::{S2DLoader, layout2::LayoutElement}, specs::{Entity, World, WorldExt,world::Builder}};
use seija::assets::{TextuteLoaderInfo};
use crate::{tests::IGameTest, core::create_image};
use seija::common::{Transform,Rect2D,Tree,TreeNode,HiddenPropagate};
use seija::render::components::{ImageRender,Mesh2D};
type DefaultBackend = seija::rendy::vulkan::Backend;
use seija::s2d::layout2::{View,Stack,LayoutAlignment,Thickness};
use seija::math::{Vector3};
#[derive(Default)]
pub struct LayoutTest {
    root:Option<Entity>,
    stack_entity:Option<Entity>,
    img01:Option<Entity>,
    index:u32
}

fn create_stack(world:&mut World,tex:Handle<Texture>,w:f32,h:f32) -> Entity {
    let mut trans = Transform::default();
    let mut img_render = ImageRender::new(Some(tex));
    trans.set_position(Vector3::new(0f32,0f32,0f32));
    img_render.set_color(0.5f32, 0.5f32, 0.5f32, 1f32);
    let mut stack = Stack::default();
    stack.spacing = 10f32;
    stack.view.margin = Thickness::new1(10f64);
    
     
    let  e = world.create_entity()
         .with(trans)
         .with(Rect2D::new(w, h, [0.5f32,0.5f32]))
         .with(img_render)
         .with(Mesh2D::default())
         .with(LayoutElement::StackLayout(stack))
         .build();
    
    e
}

fn add_img(parent:Entity,img:Handle<Texture>, world:&mut World) {
    let c0 = create_image(world, img.clone(), 70f32, 70f32, 0f32, 0f32, 0f32, 0);
    {
        let mut views = world.write_storage::<LayoutElement>();
        let mut view = View::default();
        view.hor = LayoutAlignment::Fill;
        view.ver = LayoutAlignment::Fill;
        view.size.x = 50f64;
        
        views.insert(c0, LayoutElement::ViewUnit(view)).unwrap();
    }
    Tree::add(world, c0, Some(parent));
}

impl IGameTest for LayoutTest {
    fn start(&mut self, world:&mut World) {
        let (b_jpg,white) = {
            let loader = world.write_resource::<S2DLoader>();
            let b = loader.load_sync::<_,DefaultBackend>(TextuteLoaderInfo::new_only_path("b.jpg"), world).unwrap();
            let w = loader.load_sync::<_,DefaultBackend>(TextuteLoaderInfo::new_only_path("white.png"), world).unwrap();
            (b,w)
        };
        
      
        let root0 = create_stack(world,white.clone(),640f32,480f32);
        Tree::add(world, root0, None);
       
        for _ in 0..3 {
            add_img(root0, b_jpg.clone(), world);
        }
        
       
       

        self.root = Some(root0);
        /*
        let stack_entity = create_stack(world,white.clone());
        let img01 = create_image(world, b_jpg, 200f32, 200f32, 0f32, 0f32, 0f32, 0, Some(stack_entity));
        let handle = {
            let mut view_storage = world.write_storage::<LayoutView>();
            let mut img_view = LayoutView::default();
            img_view.hor = LayoutAlignment::Fill;
            img_view.ver = LayoutAlignment::Start;
            img_view.margin = Thickness::new1(20f64);
            view_storage.insert(img01, img_view).unwrap();
            LayoutHandle::new(img01,LayoutType::View)
        };

        stack_add_child(world,stack_entity,handle);
        self.stack_entity = Some(stack_entity);
        self.img01 = Some(img01);

        self.root = Some(world.create_entity().build());
        dbg!(&self.stack_entity);
        dbg!(&self.root);*/
    }

    fn update(&mut self, world:&mut World) {
        if self.index == 50 {
          
        }

        if self.index < 100 {
            self.index +=1
        }
    }
}