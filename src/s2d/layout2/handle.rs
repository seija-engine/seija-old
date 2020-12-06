use specs::{World,WorldExt,Entity,ReadStorage};
use super::view::{LayoutView};

pub struct LayoutStorage<'a> {
    pub view:ReadStorage<'a,LayoutView>,
}

pub fn fetch_layout_storage(world:&World) -> LayoutStorage {
   let view = world.read_storage::<LayoutView>(); 
   LayoutStorage {view }
}

#[derive(Clone)]
pub enum LayoutType {
    View,
    Layout,
    StackLayout
}
#[derive(Clone)]
pub struct LayoutHandle {
    e:Entity,
    typ:LayoutType
}

impl LayoutHandle {
    pub fn new(e:Entity,typ:LayoutType) -> LayoutHandle {
        LayoutHandle {
            e,
            typ
        }
    }

    pub fn entity(&self) -> u32 {
        self.e.id()
    }

    pub fn typ(&self) -> &LayoutType {
        &self.typ
    }

    pub fn view<'a>(&self,storage:&'a LayoutStorage<'a>) -> Option<&'a LayoutView> {
        match self.typ {
            LayoutType::View => {
                storage.view.get(self.e)
            },
             _ => None
        }
    }

}