use seija::{assets::{FontAssetLoaderInfo, Handle}, render::{FontAsset, Transparent, components::TextRender, types::Texture}, s2d::{S2DLoader, layout::LayoutElement}, specs::{Entity, World, WorldExt,world::Builder}};
use seija::assets::{TextuteLoaderInfo};
use crate::{tests::IGameTest, core::create_image};
use seija::common::{Transform,Rect2D,Tree,TreeNode,HiddenPropagate};
use seija::render::components::{ImageRender,Mesh2D};
type DefaultBackend = seija::rendy::vulkan::Backend;
use seija::s2d::layout::{Orientation,LNumber,Grid,GridCell,View,Stack,LayoutAlignment,Thickness};
use seija::math::{Vector3,Vector2};
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
    img_render.set_color(0.5f32, 0.5f32, 0.9f32, 1f32);
    let mut stack = Stack::default();
    stack.spacing = 10f32;
    stack.view.margin = Thickness::new1(10f64);
    stack.orientation = Orientation::Horizontal;
     
    let  e = world.create_entity()
         .with(trans)
         .with(Rect2D::new(w, h, [0.5f32,0.5f32]))
         .with(img_render)
         .with(Mesh2D::default())
         .with(LayoutElement::StackLayout(stack))
         .with(Transparent)
         .build();
    
    e
}

fn create_grid(world:&mut World,tex:Handle<Texture>,w:f32,h:f32) -> Entity {
    let trans = Transform::default();
    let mut img_render = ImageRender::new(Some(tex));
    img_render.set_color(0.5f32, 0.5f32, 0.5f32, 1f32);
    let mut gird = Grid::default();
    gird.cols = vec![LNumber::Rate(30f32),LNumber::Rate(90f32)];
    gird.rows = vec![LNumber::Rate(50f32),LNumber::Rate(50f32)];
    
    gird.view.hor = LayoutAlignment::Fill;
    gird.view.ver = LayoutAlignment::Fill;
    //gird.view.margin = Thickness::new1(10f64);
    world.create_entity()
         .with(trans)
         .with(Rect2D::new(w, h, [0.5f32,0.5f32]))
         .with(img_render)
         .with(Mesh2D::default())
         .with(LayoutElement::GridLayout(gird))
         .build()
}

fn add_img(img:Handle<Texture>, world:&mut World) -> Entity {
    let c0 = create_image(world, img.clone(), 70f32, 70f32, 0f32, 0f32, 0f32, 0);
    {
        let mut views = world.write_storage::<LayoutElement>();
        let mut view = View::default();
        view.hor = LayoutAlignment::Fill;
        view.ver = LayoutAlignment::Fill;
        view.size.set(Vector2::new(50f64,50f64));
       
        views.insert(c0, LayoutElement::View(view)).unwrap();
    }
    c0
}

fn add_text(world:&mut World,font:Handle<FontAsset>,str:&str) -> Entity {
    let mut render = TextRender::new(Some(font));
    render.set_text(str);
    render.set_color(1f32, 0f32, 0f32, 1f32);
    render.auto_size = true;
    let mut view = View::default();
    view.use_rect_size = true;
    let elem = LayoutElement::View(view);
    world.create_entity().with(Transparent).with(elem).with(Mesh2D::default()).with(render).with(Transform::default()).with(Rect2D::new(200f32,200f32,[0.5f32,0.5f32])).build()
         
}

impl LayoutTest {
    pub fn test_stack(&mut self, world:&mut World,white:Handle<Texture>,b_jpg:Handle<Texture>) {
        let root0 = create_stack(world,white.clone(),640f32,480f32);
        Tree::add(world, root0, None);
        let font =  {
            let loader = world.write_resource::<S2DLoader>();
            loader.load_sync::<_,DefaultBackend>(FontAssetLoaderInfo::new("WenQuanYiMicroHei.ttf") ,world).unwrap()
        };

        let strs = ["A","BB","DDDDDD","1"];
        for idx in 0..4 {
            let e = add_text(world,font.clone(),strs[idx]);
            Tree::add(world, e, Some(root0));
        }
    }

    pub fn test_grid(&mut self, world:&mut World,white:Handle<Texture>,b_jpg:Handle<Texture>) {
        let root = create_grid(world, white.clone(),100f32,100f32);
        Tree::add(world, root, None);

        let img0 = add_img( white.clone(), world);
        world.write_storage::<GridCell>().insert(img0, GridCell::new(0, 0, 0, 2)).unwrap();
        Tree::add(world, img0, Some(root));

        let img1 = add_img( b_jpg.clone(), world);
        world.write_storage::<GridCell>().insert(img1, GridCell::new(1, 0, 0, 0)).unwrap();
        Tree::add(world, img1, Some(root));

        let img2 = add_img( b_jpg.clone(), world);
        world.write_storage::<GridCell>().insert(img2, GridCell::new(1, 1, 0, 0)).unwrap();
        Tree::add(world, img2, Some(root));

        //let img3 = add_img( b_jpg.clone(), world);
        //world.write_storage::<GridCell>().insert(img3, GridCell::new(1, 1, 0, 0)).unwrap();
        //Tree::add(world, img3, Some(root));
    }

    pub fn test_simple(&mut self, world:&mut World,white:Handle<Texture>) {
        let root = create_image(world, white.clone(), 70f32, 70f32, 100f32, 0f32, 0f32, 0);
       
        {
            let mut views = world.write_storage::<LayoutElement>();
            let mut view = View::default();
            view.hor = LayoutAlignment::Fill;
            view.ver = LayoutAlignment::Fill;
            view.margin = Thickness::new1(10f64);
            view.padding = Thickness::new1(10f64);
            views.insert(root, LayoutElement::View(view)).unwrap();
        };
        Tree::add(world, root, None);

        let c1 = create_image(world, white.clone(), 70f32, 70f32, 0f32, 0f32, 0f32, 3);
        {
            let mut views = world.write_storage::<LayoutElement>();
            let mut view = View::default();
            view.hor = LayoutAlignment::Fill;
            view.ver = LayoutAlignment::Fill;
            view.margin = Thickness::new1(10f64);
            view.padding = Thickness::new1(10f64);
            views.insert(c1, LayoutElement::View(view)).unwrap();
        };
        
        Tree::add(world, c1, Some(root));
    }

    
}

impl IGameTest for LayoutTest {
    fn start(&mut self, world:&mut World) {
        let (b_jpg,white) = {
            let loader = world.write_resource::<S2DLoader>();
            let b = loader.load_sync::<_,DefaultBackend>(TextuteLoaderInfo::new_only_path("b.jpg"), world).unwrap();
            let w = loader.load_sync::<_,DefaultBackend>(TextuteLoaderInfo::new_only_path("white.png"), world).unwrap();
            (b,w)
        };
        
       self.test_stack(world, white, b_jpg);
       
    }

    fn update(&mut self, world:&mut World) {
       
    }
}