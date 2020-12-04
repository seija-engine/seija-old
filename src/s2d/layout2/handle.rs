use specs::{World,WorldExt,Entity,ReadStorage};
use super::view::{LayoutView,IView};
use super::layout::Layout;

pub struct LayoutStorage<'a> {
    view:ReadStorage<'a,LayoutView>,
    layout:ReadStorage<'a,Layout>
}

pub fn fetch_layout_storage(world:&World) -> LayoutStorage {
   let view = world.read_storage::<LayoutView>(); 
   let layout = world.read_storage::<Layout>();
   LayoutStorage {view,layout }
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
    pub fn eid(&self) -> u32 {
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

    pub fn opt_view<F,R>(&self,world:&World,f:F) -> Option<R> where F:Fn(&LayoutView) -> R {
         match self.typ {
             LayoutType::View => {
               world.read_storage::<LayoutView>().get(self.e).map(|v| f(v))
             },
             LayoutType::Layout => {
                 world.read_storage::<Layout>().get(self.e).map(|v| f(v.view()))
             }
             _ => None
         }
    }
}