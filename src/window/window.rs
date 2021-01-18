use winit::{window::{WindowBuilder,WindowAttributes,Window},event::{Event,WindowEvent},event_loop::{EventLoop,ControlFlow}};
use rendy::hal::command::{ClearColor};
use winit::platform::run_return::EventLoopExtRunReturn;
pub struct WindowModule {
  window:Option<Window>,
  win_attr:Option<WindowAttributes>,
  event_loop:Option<EventLoop<()>>,
  pub clear_color:Option<ClearColor>
}

impl WindowModule {

    pub fn start(&mut self,wb:WindowBuilder) {
        let event_loop = EventLoop::new();
        //let mut win_builder = WindowBuilder::new();
        //win_builder.window.title = String::from("Seija");
        //win_builder.window.dimensions = Some(winit::dpi::LogicalSize{width:1024.into(),height:768.into()});
        self.win_attr = Some(wb.window.clone());
        let win = wb.build(&event_loop).unwrap();
        self.window = Some(win);
        self.event_loop = Some(event_loop);
    }

    pub fn update(&mut self) -> Vec<Event<()>> {
       let event_ref:&mut EventLoop<()> = self.event_loop.as_mut().unwrap();
       let mut events = Vec::new(); 
       
       event_ref.run_return(|ev:Event<'_,()>,_,control_flow| {
           if let Some(s_ev) = ev.to_static() {
            events.push(s_ev);
           }
           *control_flow = ControlFlow::Exit;
       });
      
       events
    }

    pub fn get_window(&self) -> &Window {
        
        self.window.as_ref().unwrap()
    }

    pub fn win_attr(&self) -> &WindowAttributes {
        self.win_attr.as_ref().unwrap()
    }

    pub fn set_clear_color(&mut self,clear:impl Into<ClearColor>) {
        self.clear_color = Some(clear.into());
    }

    pub fn quit(&mut self) {
        self.event_loop.take();
        self.window.take();
        println!("WindowModule quit");
    }


    pub fn has_close_event(win_events:&Vec<Event<()>>) -> bool {
        let mut has_close = false;
        for ev in win_events.iter() {
            match ev {
                Event::WindowEvent {event,..} => match event {
                 WindowEvent::CloseRequested => has_close = true,
                  _ => ()
                },
                _ => ()
            }
        };
        has_close
    }
}

impl Default for WindowModule {
    fn default() -> WindowModule {
        WindowModule {
            window:None,
            event_loop:None,
            win_attr:None,
            clear_color:None
        }
    }
}