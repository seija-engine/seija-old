use super::{Orientation};
use specs::{Component,Entity,DenseVecStorage,WriteStorage};
use crate::common::{transform::{ParentHierarchy},Rect2D,Transform};
pub struct StackPanel {
    pub orientation:Orientation,
    pub spacing:f32,
    dirty:bool
}

impl Default for StackPanel {
    fn default() -> Self {
        StackPanel {
            orientation:Orientation::Vertical,
            spacing:0f32,
            dirty:true
        }
    }
}

impl Component for StackPanel {
    type Storage = DenseVecStorage<StackPanel>;
}

impl StackPanel {
    pub fn process(&mut self,e:Entity,hierarchy:&ParentHierarchy,
                   rect_storage:&mut WriteStorage<Rect2D>,
                   t_storage:&mut WriteStorage<Transform>) {
        if self.dirty == false {
            return;
        }
        self.dirty = false;
        let childrens = hierarchy.children(e);
        let [min_x,_,_,max_y] = {
            let self_rect = rect_storage.get(e).unwrap();
            self_rect.corner_point()
        };
        if self.orientation == Orientation::Vertical {
            let mut add_y:f32 = max_y;
            for ce in childrens {
               let c_rect = rect_storage.get_mut(*ce).unwrap();
               let c_t = t_storage.get_mut(*ce).unwrap();
               let h_size = c_rect.height * c_t.scale().y;
               let cur_y = add_y - h_size;
               let offset_y = cur_y + (c_rect.anchor[1] * h_size);
               c_t.set_position_y(offset_y);
               add_y = cur_y - self.spacing;
            }
        } else {
            let mut add_x:f32 = min_x;
            for ce in childrens {
               let c_rect = rect_storage.get_mut(*ce).unwrap();
               let c_t = t_storage.get_mut(*ce).unwrap();
               let w_size = c_rect.width * c_t.scale().x;
               let cur_x = add_x + w_size;
               let offset_x = cur_x - ((1f32 - c_rect.anchor[0]) * w_size);
               c_t.set_position_x(offset_x);
               add_x = cur_x + self.spacing;
            }
        }
        
    }
}

