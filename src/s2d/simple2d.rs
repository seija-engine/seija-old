use crate::core::{AppControlFlow, IModuleBundle,Time};
use crate::window::{WindowModule,ViewPortSize};
use rendy::factory::{Factory};
use crate::render::{OutputOptions,ImageOptions,OutputColor,SpriteMeshSystem,
                    RenderOrder,RenderSystem,RenderBuilder,GraphNodeBuilder,
                    SpriteVisibilitySortingSystem,SpriteVisibility};
use crate::render::groups::{Flat2DGroupDesc,Flat2DTransparentDesc};
use rendy::hal::image::{Kind};
use rendy::graph::render::{RenderGroupDesc as _};
use rendy::hal::command::{ClearValue,ClearDepthStencil};
use rendy::hal::format::{Format};
use specs::{DispatcherBuilder,World,WorldExt};
use shrev::{EventChannel};
use crate::common::{EntityInfo,transform::{build_transform_module},Rect2D,UpdateSystem,Update};
use winit::{window::WindowBuilder};
use crate::assets::{Loader,S2DAssetPack,StorageCenter};
use crate::event::{GameEventHandle};
use crate::s2d::layout::{init_layout_system};

pub type DefaultBackend = rendy::vulkan::Backend;

pub type S2DLoader = Loader<S2DAssetPack>;

pub struct Simple2d  {
    window: WindowModule,
    render_system:Option<RenderSystem<rendy::vulkan::Backend>>,
    win_builder:Option<WindowBuilder>,
    bg_color:[f32;4],
    event_handle:GameEventHandle,
    update_system:UpdateSystem,
}

impl Simple2d  {
    pub fn new() -> Simple2d {
        Simple2d {
            window: WindowModule::default(), 
            render_system:None,
            win_builder:Some(WindowBuilder::new()),
            bg_color:[0.8f32,0.8f32,0.8f32,1.0],
            event_handle:GameEventHandle::new(),
            update_system:UpdateSystem::default()
        }
    }

    pub fn with_window(&mut self,f:impl Fn(&mut WindowBuilder)) {
        f(self.win_builder.as_mut().unwrap());
    }

    pub fn with_bg_color(&mut self,color:[f32;4]) {
        self.bg_color = color;
    }
}

impl IModuleBundle for Simple2d  {

    fn build(world:&mut World,builder:&mut DispatcherBuilder<'static,'static>) {
        world.register::<EntityInfo>();
        world.register::<Rect2D>();
        world.register::<Update>();
        build_transform_module(world,builder);
       
        builder.add(SpriteVisibilitySortingSystem::new(), &"sprite_visibility_system", &[]);
        builder.add(SpriteMeshSystem::<DefaultBackend>::new(),&"sprite_mesh",&[&"sprite_visibility_system"]);
        init_layout_system(world, builder);
        world.insert(SpriteVisibility::default());
        GameEventHandle::register(world);
    }

    fn start(&mut self,world:&mut World) {
        self.window.start(self.win_builder.take().unwrap());
        self.window.set_clear_color(rendy::hal::command::ClearColor {
            float32:self.bg_color
        });
        let mut render_system = RenderSystem::new(world);
        let win_surface = world.fetch_mut::<Factory<rendy::vulkan::Backend>>().create_surface(self.window.get_window()).unwrap();
        let win_size = self.window.win_attr().inner_size.unwrap().to_logical(1f64);
        world.insert(ViewPortSize::new(win_size.width ,win_size.height));
        self.event_handle.set_view_size((win_size.width,win_size.height));
        let kind = Kind::D2(win_size.width as u32, win_size.height as u32, 1, 1);
        let depth = Some(ImageOptions {
            kind,
            levels: 1,
            format: Format::D32Sfloat,
            clear: Some(ClearValue { depth_stencil:ClearDepthStencil {
                depth:1.0, stencil:0
            } }),
         });

        let node2d = GraphNodeBuilder::new().with_name("flat2d")
                                           .with_group(RenderOrder::Opaque,Flat2DGroupDesc::new().builder())
                                           .with_group(RenderOrder::Transparent, Flat2DTransparentDesc::new().builder())
                                           .build(OutputOptions {
                                                     colors: vec![OutputColor::Surface(win_surface,self.window.clear_color.map(|c|
                                                        ClearValue {color:c}
                                                     ))],
                                                     depth: depth
                                                  });
        render_system.build(RenderBuilder::new().with_root_node(node2d), world);

        S2DAssetPack::register_all_storage(world);
        world.insert(StorageCenter::default());
        world.insert(Loader::<S2DAssetPack>::default());
        
        self.render_system = Some(render_system);
    }

    fn update(&mut self,world:&mut World) {
        let win_events = self.window.update();
        self.event_handle.fire_event(&win_events,world);
        if WindowModule::has_close_event(&win_events) {
            world.write_resource::<EventChannel<AppControlFlow>>().single_write(AppControlFlow::Quit);
        };
        let dt = world.read_resource::<Time>().delta_seconds();
        self.update_system.update(dt, world);
        self.render_system.as_mut().unwrap().update(world);
    }

    fn quit(&mut self,world:&mut World) {
       
        if let Some(render_system) = self.render_system.take() {
            render_system.dispose(world);
        }
        self.window.quit();
    }
}
