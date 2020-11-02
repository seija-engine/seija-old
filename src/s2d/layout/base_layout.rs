use super::{HorizontalAlign,VerticalAlign,LayoutRect};
use specs::{Component,DenseVecStorage,WriteStorage,Entity};
use crate::common::transform::{ParentHierarchy};
use crate::common::{Transform,Rect2D};
pub struct BaseLayout {
    pub horizontal_align:HorizontalAlign,
    pub vertical_align:VerticalAlign,
    pub margin:LayoutRect,
    pub padding:LayoutRect,
    dirty:bool
}

impl Default for BaseLayout {
    fn default() -> Self {
        BaseLayout {
            horizontal_align:HorizontalAlign::default(),
            vertical_align:VerticalAlign::default(),
            dirty:true,
            margin:LayoutRect::default(),
            padding:LayoutRect::default()
        }
    }
}

impl Component for BaseLayout {
    type Storage = DenseVecStorage<BaseLayout>;
}

impl BaseLayout {

    pub fn process(e:Entity,hierarchy:&ParentHierarchy,
                   layout_storage:&mut WriteStorage<BaseLayout>,
                   t_storage:&mut WriteStorage<Transform>,rect_storage:&mut WriteStorage<Rect2D>) {
        if let Some(base_layout) = layout_storage.get_mut(e) {
            base_layout.calc_layout(e,hierarchy.parent(e),t_storage,rect_storage);
        }
        let childrens = hierarchy.children(e);
        for ceid in childrens {
            BaseLayout::process(*ceid, hierarchy, layout_storage,t_storage,rect_storage);
        }
    }

    pub fn calc_layout(&mut self,e:Entity,may_parent:Option<Entity>,t_storage:&mut WriteStorage<Transform>,rect_storage:&mut WriteStorage<Rect2D>) {
        if self.dirty == false || may_parent.is_none() || t_storage.get(may_parent.unwrap()).is_none() ||  rect_storage.get(may_parent.unwrap()).is_none() {
            return;
        }
        self.dirty = false;
        let parent = may_parent.unwrap();
        let (pw,ph,p_ponts) = {
            let p_rect = rect_storage.get(parent).unwrap();
            (p_rect.width,p_rect.height,p_rect.corner_point())
        };
        let self_rect = rect_storage.get_mut(e).unwrap();
        let t_self = t_storage.get_mut(e).unwrap();
        let [min_x,max_x,min_y,max_y] = p_ponts;
       
        match self.horizontal_align {
            HorizontalAlign::Left => {
                t_self.set_position_x(min_x);
            },
            HorizontalAlign::Center => {
                t_self.set_position_x((min_x + max_x) / 2f32);
            },
            HorizontalAlign::Right => {
                t_self.set_position_x(max_x);
            },
            HorizontalAlign::Stretch => {
                t_self.set_position_x((min_x + max_x) / 2f32);
                self_rect.width = pw;
                
            }
        }
        
        match self.vertical_align {
            VerticalAlign::Top => {
                t_self.set_position_y(max_y);
            },
            VerticalAlign::Center => {
                t_self.set_position_y((max_y + min_y) / 2f32);
            },
            VerticalAlign::Bottom => {
                t_self.set_position_y(min_y);
            }
            VerticalAlign::Stretch => {
                t_self.set_position_y((max_y + min_y) / 2f32);
                self_rect.height = ph;
            }
        }
    }
}