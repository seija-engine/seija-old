use shred::{ReadExpect, System};
use specs::{Entities, Join, WriteStorage};

use crate::{common::Tree, render::components::TextRender};

use super::raw_input::RawInput;



pub type UIUpdateData<'a> = (
    Entities<'a>,
    ReadExpect<'a,Tree>,
    WriteStorage<'a,TextRender>,
    WriteStorage<'a,RawInput>
);

#[derive(Default)]
pub struct UIUpdateSystem {
 
}

impl<'a> System<'a> for UIUpdateSystem {
    type SystemData = UIUpdateData<'a>;
    fn run(&mut self,mut data: Self::SystemData) {
        let inputs = &mut data.3;
        for input in inputs.join() {
            if input.is_focus {
                input.update(&mut data.2)
            }
        }
    }
}