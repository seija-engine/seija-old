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
       let content_size:Vector2<f64> = self.view.calc_content_size(size,rects.get(entity).unwrap());
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
                   let  (mut child_size,
                             hor,
                             ver,
                             mh,
                             mv) = elem.fview(|v| (v.get_size(rects.get(*centity).unwrap()),v.hor,v.ver,v.margin.horizontal(),v.margin.vertical()));
                   match self.orientation {
                       Orientation::Horizontal => {
                           child_size.y = inner_size.y;
                           if child_size.x > inner_size.x {
                            child_size.x = inner_size.x;
                           }
                           if ver == LayoutAlignment::Fill {
                               elem.fview(|v| v.size.set(Vector2::new(child_size.x,child_size.y - mv)));
                           }
                           let msize:Vector2<f64> = elem.measure(*centity, child_size, rects, tree_nodes, elems,cells);
                           ret_size.x += msize.x;
                           ret_size.x += self.spacing as f64;
                       },
                       Orientation::Vertical => {
                            child_size.x = inner_size.x;
                            if child_size.y > inner_size.y {
                                child_size.y = inner_size.y;
                            }
                            if hor == LayoutAlignment::Fill {
                                elem.fview(|v| v.size.set(Vector2::new(child_size.x- mh,child_size.y)));
                            }
                            let msize:Vector2<f64> = elem.measure(*centity, child_size, rects, tree_nodes, elems,cells);
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
       
       let mut add_number  = match self.orientation {
           Orientation::Horizontal => self.view.padding.left as f32,
           Orientation::Vertical =>  -self.view.padding.top as f32,
       };
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
                            LayoutAlignment::Start => new_pos.y = 0f32 - self.view.padding.top as f32,
                            LayoutAlignment::Center => {
                                new_pos.y = -height * 0.5f32 + child_height * 0.5f32;
                            },
                            LayoutAlignment::End => {
                                new_pos.y = -height + child_height + self.view.padding.bottom as f32;
                            },
                            LayoutAlignment::Fill => {
                                new_pos.y = 0f32 - self.view.padding.top as f32;
                                //rects.get_mut(*centity).map(|v| {
                                //    v.height = height - self.view.padding.vertical() as f32;
                                //});
                            },
                        }
                        let mh = elem.fview(|v| {v.pos.set(new_pos);v.margin.horizontal() } );
                        
                        elem.arrange(*centity, size, rects, tree_nodes, elems, trans, child_origin,cells);
                        add_number += child_width + mh as f32;
                        add_number += self.spacing;
                    },
                    Orientation::Vertical => {
                        new_pos.y = add_number;
                        match elem.fview(|v| v.hor) {
                            LayoutAlignment::Start => new_pos.x = 0f32 + self.view.padding.left as f32,
                            LayoutAlignment::Center => {
                                new_pos.x = width * 0.5f32 - (child_width * 0.5f32);
                            },
                            LayoutAlignment::End => {
                                new_pos.x = width - child_height - self.view.padding.right as f32;
                            },
                            LayoutAlignment::Fill => {
                                new_pos.x = 0f32 + self.view.padding.left as f32;
                                //rects.get_mut(*centity).map(|v| {
                                //    v.width = width - self.view.padding.horizontal() as f32;
                                //});
                            },
                        }
                        
                        let mv = elem.fview(|v| {v.pos.set(new_pos); v.margin.vertical() });
                        elem.arrange(*centity, size, rects, tree_nodes, elems, trans, child_origin,cells);
                        add_number -= child_height + mv as f32;
                        add_number -= self.spacing;
                    }
                }
                
            }
         }
       }
        
   }
}
