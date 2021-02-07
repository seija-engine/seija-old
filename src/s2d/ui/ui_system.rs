use std::borrow::Borrow;

use shred::{ReadExpect, System};
use specs::{Entities, Join, ReadStorage, WriteStorage};

use crate::{common::{TreeNode}, core::Time, render::components::{Mesh2D, TextRender}};

use super::raw_input::RawInput;



pub type UIUpdateData<'a> = (
    Entities<'a>,
    ReadStorage<'a,TreeNode>,
    ReadExpect<'a,Time>,
    WriteStorage<'a,TextRender>,
    WriteStorage<'a,RawInput>,
    WriteStorage<'a,Mesh2D>
);

#[derive(Default)]
pub struct UIUpdateSystem {
 
}

impl<'a> System<'a> for UIUpdateSystem {
    type SystemData = UIUpdateData<'a>;
    fn run(&mut self,mut data: Self::SystemData) {
        let t = data.2.borrow().delta_seconds();
        let inputs = &mut data.4;
        
        for  input in inputs.join() {
            if input.is_focus {
                input.update( &mut data.3, t);
            }
        }
    }
}