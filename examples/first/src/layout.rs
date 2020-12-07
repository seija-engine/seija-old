use seija::{assets::Handle, render::types::Texture, s2d::{S2DLoader}, specs::{Entity, World, WorldExt,world::Builder}};
use seija::assets::{TextuteLoaderInfo};
use crate::{tests::IGameTest, core::create_image};
use seija::common::{Transform,Rect2D,transform::Parent,Tree,TreeNode,HiddenPropagate};
use seija::render::components::{ImageRender,Mesh2D};
type DefaultBackend = seija::rendy::vulkan::Backend;
use seija::s2d::layout2::{LayoutView,LayoutHandle,StackLayout,LayoutAlignment,Thickness};
#[derive(Default)]
pub struct LayoutTest {
    root:Option<Entity>,
    stack_entity:Option<Entity>,
    img01:Option<Entity>,
    index:u32
}

fn create_stack(world:&mut World,tex:Handle<Texture>,w:f32,h:f32) -> Entity {
    world.create_entity()
         .with(Transform::default())
         .with(Rect2D::new(w, h, [0.5f32,0.5f32]))
         .with(ImageRender::new(Some(tex)))
         .with(Mesh2D::default())
         .with(StackLayout::default())
         .build()
}


impl IGameTest for LayoutTest {
    fn start(&mut self, world:&mut World) {
        let (b_jpg,white) = {
            let loader = world.write_resource::<S2DLoader>();
            let b = loader.load_sync::<_,DefaultBackend>(TextuteLoaderInfo::new_only_path("b.jpg"), world).unwrap();
            let w = loader.load_sync::<_,DefaultBackend>(TextuteLoaderInfo::new_only_path("white.png"), world).unwrap();
            (b,w)
        };
        
      
        let root0 = create_stack(world,white,800f32,600f32);
        Tree::add(world, root0, None);
        let c0 = create_image(world, b_jpg.clone(), 100f32, 100f32, 0f32, 0f32, 0f32, 0, None);
        Tree::add(world, c0, Some(root0));

       
        let c1 = create_image(world, b_jpg, 100f32, 100f32, 120f32, 0f32, 0f32, 0, None);
        Tree::add(world,c1,Some(root0));

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
            {
                let mut s_hidden = world.write_storage::<HiddenPropagate>();
                s_hidden.insert(self.root.unwrap(), HiddenPropagate).unwrap();
            };
        }

        if self.index < 100 {
            self.index +=1
        }
    }
}