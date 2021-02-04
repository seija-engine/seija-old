use std::borrow::BorrowMut;

use crate::tests::IGameTest;
use seija::{assets::FontAssetLoaderInfo, common::Tree, render::{Transparent, components::{ImageRender, Mesh2D}}, s2d::{layout::Thickness, ui::raw_input::RawInput}, specs::WriteStorage};
use seija::{
    assets::TextuteLoaderInfo,
    common::{Rect2D, Transform},
    event::cb_event::CABEventRoot,
    s2d::{
        layout::{ContentView, LayoutElement},
        S2DLoader,
    },
    shred::World,
    specs::{Builder, WorldExt},
};
type DefaultBackend = seija::rendy::vulkan::Backend;
#[derive(Default)]
pub struct InputTest {}

impl IGameTest for InputTest {
    fn start(&mut self, world: &mut World) {
        let (font, white) = {
            let loader = world.write_resource::<S2DLoader>();
            let b = loader.load_sync::<_,DefaultBackend>(FontAssetLoaderInfo::new("WenQuanYiMicroHei.ttf"),world).unwrap();
            let w = loader.load_sync::<_, DefaultBackend>(TextuteLoaderInfo::new_only_path("white.png"), world,).unwrap();
            (b, w)
        };

        let root = world.create_entity()
                               .with(CABEventRoot::default())
                               .with(Transform::default())
                               .with(Rect2D::default())
                               .with(LayoutElement::ContentView(ContentView::default()))
                               .build();
        
        Tree::add(world, root, None);
        let input_entity = world.create_entity().build();
        Tree::add(world, input_entity, Some(root));
        RawInput::attach_new(input_entity,Some(font), world);
        
        let mut elems:WriteStorage<LayoutElement> = world.write_storage::<LayoutElement>();
        let le = elems.get_mut(input_entity).unwrap();
        le.fview_mut(|view| {
            view.margin = Thickness::new1(100f64);
        });
        world.write_storage::<ImageRender>().insert(input_entity, ImageRender::new(Some(white))).unwrap();
        world.write_storage::<Transparent>().insert(input_entity, Transparent::default()).unwrap();
        world.write_storage::<Mesh2D>().insert(input_entity, Mesh2D::default()).unwrap();
    }

    fn update(&mut self, world: &mut World) {}
}
