use specs::{System,ReadExpect,Entities,WriteStorage,ReadStorage,Join};
use crate::window::{ViewPortSize};
use crate::common::{Transform,Rect2D};
use super::base_layout::{BaseLayout};
use super::stack_panel::{StackPanel};
use super::ScreenScaler;
use crate::common::transform::{ParentHierarchy,Parent};
pub struct LayoutSystem {
}

impl Default for LayoutSystem {
    fn default() -> Self {
        LayoutSystem { }
    }
}

impl<'a> System<'a> for LayoutSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a,ScreenScaler>,
        WriteStorage<'a,Transform>,
        WriteStorage<'a,Rect2D>,
        WriteStorage<'a,BaseLayout>,
        WriteStorage<'a,StackPanel>,
        ReadStorage<'a,Parent>,
        ReadExpect<'a,ViewPortSize>,
        ReadExpect<'a,ParentHierarchy>
    );
    fn run(&mut self, (entities,mut screen_scaleres,mut trans,mut rects,mut base_layouts,mut stacks,parents,view_port,hierarchy): Self::SystemData) {
        for (_e,screen_scaler,t,rect) in (&entities,&mut screen_scaleres,&mut trans,&mut rects).join() {
            screen_scaler.process(&view_port,t,rect);
        }

        for(e,_) in (&entities,!&parents).join() {
            BaseLayout::process(e, &hierarchy,&mut base_layouts,&mut trans,&mut rects);
        }
        
        for(e,stack) in (&entities,&mut stacks).join() {
            stack.process(e,&hierarchy,&mut rects,&mut trans);
        }
    }
}

