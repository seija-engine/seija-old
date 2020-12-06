use crate::common::{Tree,TreeEvent};
use specs::{System,World,ReadExpect};
use shrev::{ReaderId};

pub struct LayoutSystem {
   ev_tree:ReaderId<TreeEvent>
}

impl LayoutSystem {
    pub fn new(world:&mut World) -> LayoutSystem {
        let tree = world.get_mut::<Tree>().unwrap();
        LayoutSystem {
           ev_tree:tree.channel.register_reader()
        }
    }
}

impl<'a> System<'a> for LayoutSystem {
    type SystemData = ReadExpect<'a,Tree>;
    fn run(&mut self, tree: Self::SystemData) {
       for ev in tree.channel.read(&mut self.ev_tree) {
           match ev {
            TreeEvent::Add(p,e) => {
                
            },
            TreeEvent::Remove(p,c) => {
                
            }
           }
       }
    }
}