use specs::{Component, DenseVecStorage, Entity, ReadStorage, WriteStorage};
use crate::common::{Rect2D, Transform, TreeNode};
use nalgebra::{Vector2,Vector3};
use super::{IView, LayoutElement, View,LayoutAlignment,GridCell};

impl Component for Stack {
    type Storage = DenseVecStorage<Stack>;
}

pub enum Orientation {
    Horizontal,
    Vertical,
}

impl From<u32> for Orientation {
    fn from(n: u32) -> Orientation {
        match n {
            0 => Orientation::Horizontal,
            _ => Orientation::Vertical
        }
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Orientation::Horizontal
    }
}

#[derive(Default)]
pub struct Stack {
   pub view:View,
   pub orientation: Orientation,
   pub spacing:f32,
   pub over_hide:bool
}

impl IView for Stack {
   fn measure(&self,entity:Entity, size:Vector2<f64>
              ,rects:&mut WriteStorage<Rect2D>
              ,tree_nodes:&ReadStorage<TreeNode>
              ,elems:&WriteStorage<LayoutElement>
              ,cells:&ReadStorage<GridCell>) -> Vector2<f64> {
       let mut ret_size:Vector2<f64> = size;
       let content_size:Vector2<f64> = self.view.calc_content_size(size);
       rects.get_mut(entity).map(|rect| {
              rect.width = content_size.x as f32;
              rect.height = content_size.y as f32;
       });
       
       let inner_size:Vector2<f64> = Vector2::new(content_size.x - self.view.padding.horizontal(),
                                                  content_size.y - self.view.padding.vertical());
       let m_child = tree_nodes.get(entity).map(|v| &v.children);
       match self.orientation {
           Orientation::Horizontal => ret_size.x = 0f64,
           Orientation::Vertical => ret_size.y = 0f64
       }
       if let Some(child) = m_child {
           for centity in child {
               if let Some(elem) = elems.get(*centity) {
                   let mut csize:Vector2<f64> = elem.fview(|v| v.size.get());
                   match self.orientation {
                       Orientation::Horizontal => {
                           csize.y = inner_size.y;
                           if csize.x > inner_size.x {
                               csize.x = inner_size.x;
                           }
                           let msize:Vector2<f64> = elem.measure(*centity, csize, rects, tree_nodes, elems,cells);
                           ret_size.x += msize.x;
                           ret_size.x += self.spacing as f64;
                       },
                       Orientation::Vertical => {
                            csize.x = inner_size.x;
                            if csize.y > inner_size.y {
                                csize.y = inner_size.y;
                            }
                            let msize:Vector2<f64> = elem.measure(*centity, csize, rects, tree_nodes, elems,cells);
                            ret_size.y += msize.y;
                            ret_size.y += self.spacing as f64;
                       }
                   }
               }
           }
       }
       if self.over_hide {
           size
       } else {
           ret_size
       }
   }

   fn arrange(&self, entity:Entity, size:Vector2<f64>
    , rects:&mut WriteStorage<Rect2D>
    , tree_nodes:&ReadStorage<TreeNode>
    , elems:&WriteStorage<LayoutElement>
    , trans:&mut WriteStorage<Transform>
    , origin:Vector3<f32>
    , cells:&ReadStorage<GridCell>) {
       self.view.arrange(entity, size, rects, tree_nodes, elems, trans, origin,cells);
       let child_origin = self.view.calc_orign(entity, rects);
       let (width,height) = {
           let rect = rects.get(entity).unwrap();
           (rect.width,rect.height)
       };
       let m_child = tree_nodes.get(entity).map(|v| &v.children);
       let mut add_number = 0f32;
       if let Some(child) = m_child {
         for centity in child {
            if let Some(elem) = elems.get(*centity) {
                let (child_width,child_height) = {
                    let rect = rects.get(*centity).unwrap();
                    (rect.width,rect.height)
                };
                let mut new_pos:Vector3<f32> = Vector3::default();
                match self.orientation {
                    Orientation::Horizontal => {
                        new_pos.x = add_number;
                        match elem.fview(|v| v.ver) {
                            LayoutAlignment::Start => new_pos.y = 0f32,
                            LayoutAlignment::Center => {
                                new_pos.y = -height * 0.5f32 + child_height * 0.5f32;
                            },
                            LayoutAlignment::End => {
                                new_pos.y = -height + child_height;
                            },
                            LayoutAlignment::Fill => {
                                new_pos.y = 0f32;
                                rects.get_mut(*centity).map(|v| {
                                    v.height = height;
                                });
                            },
                        }
                        elem.fview(|v| v.pos.set(new_pos));
                        
                        elem.arrange(*centity, size, rects, tree_nodes, elems, trans, child_origin,cells);
                        add_number += child_width;
                        add_number += self.spacing;
                    },
                    Orientation::Vertical => {
                        new_pos.y = add_number;
                        match elem.fview(|v| v.hor) {
                            LayoutAlignment::Start => new_pos.x = 0f32,
                            LayoutAlignment::Center => {
                                new_pos.x = width * 0.5f32 - (child_width * 0.5f32);
                            },
                            LayoutAlignment::End => {
                                new_pos.x = width - child_height;
                            },
                            LayoutAlignment::Fill => {
                                new_pos.x = 0f32;
                                rects.get_mut(*centity).map(|v| {
                                    v.width = width;
                                });
                            },
                        }
                        
                        elem.fview(|v| v.pos.set(new_pos));
                        elem.arrange(*centity, size, rects, tree_nodes, elems, trans, child_origin,cells);
                        add_number -= child_height;
                        add_number -= self.spacing;
                    }
                }
                
            }
         }
       }
        
   }
}
/*

impl StackLayout {
    pub fn measure(
        &self,
        entity: Entity,
        size: Vector2<f64>,
        rect2d: &mut WriteStorage<Rect2D>,
        views: &mut WriteStorage<LayoutView>,
        tree_nodes: &ReadStorage<TreeNode>,
    ) -> Vector2<f64> {
        let mut self_size = {
            let rect2d = rect2d.get(entity).unwrap();
            Vector2::new(rect2d.width as f64, rect2d.height as f64)
        };
        if let Some(view) = views.get(entity) {
            self_size = view.measure(entity, size, views, rect2d);
        }
        let mut sub_size = self_size;
        let childs = tree_nodes.get(entity).unwrap().children.clone();
        for child in childs.iter() {
            let view = views.get(*child).unwrap();
            let old_size = rect2d.get_mut(*child).unwrap();
            match self.orientation {
                Orientation::Horizontal => {
                    sub_size.x = old_size.width as f64;
                }
                Orientation::Vertical => {
                    sub_size.y = old_size.height as f64;
                }
            }
            view.measure(*child, sub_size, views, rect2d);
        }

        let self_rect = rect2d.get_mut(entity).unwrap();
        self_rect.height = self_size.y as f32;
        self_rect.width = self_size.x as f32;

        self_size
    }

    pub fn arrange(
        &self,
        entity: Entity,
        rect2d: &mut WriteStorage<Rect2D>,
        tree_nodes: &ReadStorage<TreeNode>,
        trans: &mut WriteStorage<Transform>,
        views: &WriteStorage<LayoutView>,
    ) {
        let [lx, rx, by, ty] = rect2d.get(entity).unwrap().corner_point();

        let childs = tree_nodes
            .get(entity)
            .map(|v| v.children.clone())
            .unwrap_or(vec![]);
        match self.orientation {
            Orientation::Horizontal => {
                let mut x = lx;
                for child in childs {
                    let child_w = rect2d.get(child).map(|r| r.width).unwrap_or(0f32);
                    let ct = trans.get_mut(child).unwrap();
                    let lx = rect2d.get(child).unwrap().left().abs();
                    ct.set_position_x(x + lx);
                    match views.get(child).unwrap().ver {
                        LayoutAlignment::Start => {
                            ct.set_position_y(ty);
                        },
                        LayoutAlignment::Fill => {
                            
                        },
                        LayoutAlignment::End => {
                            ct.set_position_y(by);
                        },
                        LayoutAlignment::Center => {
                            
                        }
                    }
                    x += child_w + self.specing;
                }
            }
            Orientation::Vertical => {}
        }
    }

   
}
*/