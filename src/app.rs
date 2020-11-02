use crate::core::{ 
    UpdateLimiter,LimitSetting,
    Stopwatch,Time,IModuleBundle,
    AppControlFlow};
use specs::{World,WorldExt,DispatcherBuilder,Dispatcher};
use shrev::{ReaderId,EventChannel};
use crate::core::{IGame};
use rayon::{ThreadPoolBuilder};
#[cfg(feature = "profiler")]
use thread_profiler::{profile_scope, register_thread_with_profiler, write_profile};
use std::sync::{Arc};

pub struct App<G,T:IModuleBundle> where G:IGame {
    update_limiter:UpdateLimiter,
    is_runing:bool,
    module_bundle:T,
    game:G,
    world:World,
    dispatch:Dispatcher<'static,'static>,
    app_reader:ReaderId<AppControlFlow>
}

impl<G,T> App<G,T> where T:IModuleBundle,G:IGame {

    pub fn new(g:G,ctx:T,dispatch:Dispatcher<'static,'static>,world: World) -> Self {
       #[cfg(feature = "profiler")]
       register_thread_with_profiler();
       let mut app_event_channel = EventChannel::new();
       let mut app = App {
            is_runing: false,
            update_limiter: Default::default(),
            module_bundle:ctx,
            game:g,
            world:world,
            dispatch: dispatch,
            app_reader: app_event_channel.register_reader()
       };
       app.world.insert(Time::default());
       app.world.insert(Stopwatch::new());
       app.world.insert(app_event_channel);
       app
    }

    pub fn run(&mut self) {
        self.is_runing = true;
        self.module_bundle.start(&mut self.world);
        self.update_limiter.start();
        self.game.start(&mut self.world);
        self.world.write_resource::<Stopwatch>().start();
        let world = &mut self.world;
        while self.is_runing {
            #[cfg(feature = "profiler")]
            profile_scope!("dispatch");
            self.dispatch.dispatch(&world);
            
            #[cfg(feature = "profiler")]
            profile_scope!("maintain");
            world.maintain();

            self.game.update(world);
            self.module_bundle.update(world);
            
            let appev_iter = world
            .read_resource::<EventChannel<AppControlFlow>>()
            .read(&mut self.app_reader).map(|e| *e).collect::<Vec<_>>();
            for appev in appev_iter {
                match appev {
                    AppControlFlow::Quit => self.is_runing = false,
                    _ => (),
                }
            }

            

            self.update_limiter.wait();
            let elapsed = world.write_resource::<Stopwatch>().elapsed();
            world.write_resource::<Time>().set_delta_time(elapsed);
            world.write_resource::<Time>().inc_frame_number();
            world.write_resource::<Stopwatch>().stop();
            world.write_resource::<Stopwatch>().restart();
        }
        self.game.quit(&mut self.world);
        self.module_bundle.quit(&mut self.world);
        #[cfg(feature = "profiler")]
        write_profile("thread_profile.json");
    }

    pub fn close(&mut self) {
        self.is_runing = false;
    }
}

pub struct AppBuilder {
    update_limiter:UpdateLimiter,
}

impl Default for AppBuilder {
    fn default() -> AppBuilder {
        AppBuilder {
            update_limiter:Default::default()
        }
    }
}

impl AppBuilder {

    pub fn new() -> AppBuilder {
        Default::default()
    }

    pub fn with_update_limiter(mut self,setting:LimitSetting) -> Self {
        self.update_limiter = UpdateLimiter::new(setting);
        self
    }
    
    pub fn build<G:IGame,T:IModuleBundle>(self,ctx:T,g:G) -> App<G,T> {
        let mut world = World::new();
        let thread_pool_builder = ThreadPoolBuilder::new().num_threads(2);
        #[cfg(feature = "profiler")]
        let thread_pool_builder = thread_pool_builder.start_handler(|_index| {
            register_thread_with_profiler();
        });
        let thread_pool = thread_pool_builder.build().map(Arc::new).unwrap();
        let mut dispatch_builder = DispatcherBuilder::new().with_pool(thread_pool.clone());
        world.insert(thread_pool);
        
        
        
        T::build(&mut world,&mut dispatch_builder);
        let dispatch = dispatch_builder.build();
        let mut app = App::new(g,ctx,dispatch,world);
        app.update_limiter = self.update_limiter;
        app
    }
}


