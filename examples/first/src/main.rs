use input::InputTest;
use seija::window::{ViewPortSize};
use seija::common::{Transform};
use seija::s2d::{S2DLoader};
use seija::core::IGame;
use seija::render::{ActiveCamera,Camera};
use seija::specs::{World,WorldExt,world::Builder};
use seija::s2d::{Simple2d};
use seija::app::{AppBuilder};
use seija::core::{LimitSetting};
use seija::win::dpi::{Size,LogicalSize};

pub mod core;
mod tests;
mod layout;
mod input;
use tests::{FontTest,IGameTest,SpriteTest,EventTest,UITest};


#[derive(PartialEq,Eq)]
pub enum GameState {
    OnOpen,
    LoadAsset
}
pub struct TestGame{
    test:Box<dyn IGameTest>
}

impl TestGame {
    pub fn new(test_name:&str) -> Self {
        let box_test:Box<dyn IGameTest> = {
            match test_name {
                "font" => Box::new(FontTest::default()),
                "sprite" => Box::new(SpriteTest::default()),
                "event"  => Box::new(EventTest::default()),
                "ui"     => Box::new(UITest::default()),
                "layout" => Box::new(layout::LayoutTest::default()),
                "input" => Box::new(input::InputTest::default()),
                _ => { panic!("error test name") }
            }
        };
        TestGame {
            test:box_test
        }
    }



}

impl IGame for TestGame {
    fn start(&mut self,world:&mut World) {
        let camera_transform = Transform::default();
        //camera_transform.set_position(Vector3::new(1f32,2f32,3f32));
        let (w,h) = {
           let view_port = world.fetch::<ViewPortSize>();
           (view_port.width() as f32,view_port.height() as f32)
        };
        let entity = world.create_entity().with(camera_transform).with(Camera::standard_2d(w, h)).build();
        world.insert(ActiveCamera {entity : Some(entity) });
        world.fetch::<S2DLoader>().env().set_fs_root("./res/");
        self.test.start(world);
    }

    fn update(&mut self,world:&mut World) {
        self.test.update(world);
    }

    fn quit(&mut self,world: &mut World){
        self.test.quit(world);
    }
}


fn main() {
   
    let test_game = TestGame::new("input");
    let mut s2d = Simple2d::new();
    s2d.with_window(|wb| {
        wb.window.title = String::from("Seija Runing");
        wb.window.inner_size = Some(Size::Logical(LogicalSize::new(320f64,240f64)))
       
    });
    s2d.with_bg_color([0.6f32,0.6f32,0.6f32,1f32]);
    let mut app = AppBuilder::new().with_update_limiter(LimitSetting::Sleep(30)).build(s2d,test_game);
    app.run();
}