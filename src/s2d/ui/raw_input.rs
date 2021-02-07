use crate::{assets::{Handle}, common::{AnchorAlign, Rect2D, Transform, Tree}, event::{global::GlobalEventNode, EventNode, GameEvent, GameEventCallBack, GameEventType}, render::{
        components::{Mesh2D, TextRender},
        FontAsset, Transparent,
    }, s2d::layout::{ContentView, LayoutElement, View}};
use specs::{
    world::World, Builder, Component, DenseVecStorage, Entity, ReadStorage, WorldExt, WriteStorage,
};
/*
Entity(RawInput)
  Entity(TextRender)

*/

pub struct RawInput {
    pub text_value:String,
    pub is_focus: bool,
    pub label: Entity,
    pub time:f32,
    pub cursor_idx:usize,
    pub show_cursor:bool
}

impl Component for RawInput {
    type Storage = DenseVecStorage<Self>;
}

pub struct RawInputCallBack {
    pub entity:Entity,
    pub label:Entity
}

impl GameEventCallBack for RawInputCallBack {
    fn run(&self, ev:&GameEvent, world:&mut World) {
        let mut raw_inputs:WriteStorage<RawInput> = world.write_storage::<RawInput>();
        let mut texts:WriteStorage<TextRender> = world.write_storage::<TextRender>();
        let raw_input = raw_inputs.get_mut(self.entity).unwrap();
        if !raw_input.is_focus {
            return;
        }
        match ev {
            GameEvent::RecvChar(chr) => {
                if *chr == '\u{8}' {
                        let rm_idx = raw_input.cursor_idx as i32 - 1;
                        if rm_idx >= 0 {
                            raw_input.text_value = string_rm_idx(&raw_input.text_value,raw_input.cursor_idx);
                            raw_input.cursor_idx -= 1;
                        }
                    
                } else {
                    raw_input.text_value = string_insert_idx(&raw_input.text_value, raw_input.cursor_idx, *chr);
                    raw_input.cursor_idx += 1;
                }

                let text = texts.get_mut(self.label).unwrap();
                let  set_s = string_insert_idx(&raw_input.text_value, raw_input.cursor_idx, '|');
                text.set_text_string(set_s);
            },
            GameEvent::KeyBoard(code,b) => {
                match code {
                    70 if raw_input.cursor_idx > 0 && *b => {
                        raw_input.cursor_idx -= 1;
                        raw_input.time = 0f32;
                        raw_input.update_show_cursor(true,&mut texts);
                    },
                    72 if raw_input.cursor_idx < raw_input.char_len() && *b => {
                        raw_input.cursor_idx += 1;
                        raw_input.time = 0f32;
                        raw_input.update_show_cursor(true,&mut texts);
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        
        
        
    }
}

impl RawInput {
    pub fn new(label: Entity) -> RawInput {
        RawInput {
            text_value:String::default(),
            label,
            is_focus: false,
            show_cursor: false,
            time:0f32,
            cursor_idx:0
        }
    }

    pub fn attach_new(entity: Entity, font: Option<Handle<FontAsset>>, world: &mut World) {
        {
            let raw_inputs: ReadStorage<RawInput> = world.read_storage::<RawInput>();
            if raw_inputs.contains(entity) {
                return;
            }
        };

        let context_view = LayoutElement::ContentView(ContentView::default());
        world.write_storage::<LayoutElement>().insert(entity, context_view).unwrap();
        {
            let mut trans: WriteStorage<Transform> = world.write_storage::<Transform>();
            let mut rects: WriteStorage<Rect2D> = world.write_storage::<Rect2D>();
            let mut elems: WriteStorage<LayoutElement> = world.write_storage::<LayoutElement>();
            let mut evnodes: WriteStorage<EventNode> = world.write_storage::<EventNode>();
            if !trans.contains(entity) {
                trans.insert(entity, Transform::default()).unwrap();
            }
            if !rects.contains(entity) {
                rects.insert(entity, Rect2D::default()).unwrap();
            }
            if !elems.contains(entity) {
                elems
                    .insert(entity, LayoutElement::ContentView(ContentView::default()))
                    .unwrap();
            }
            let mut ev_node = EventNode::default();
            ev_node.register(true, crate::event::GameEventType::TouchStart, |e, w| {
                let mut raw_inputs: WriteStorage<RawInput> = w.write_storage::<RawInput>();
                let raw_input = raw_inputs.get_mut(e).unwrap();
                raw_input.is_focus = true;
            });
            evnodes.insert(entity, ev_node).unwrap();

            
        };

        let mut text_render = TextRender::new(font);
        text_render.set_anchor(AnchorAlign::Left);
        text_render.set_font_size(20);
        let mut rect2d = Rect2D::default();
        rect2d.set_anchor([0f32, 0.5f32]);
        let mut ev_node = EventNode::default();
        ev_node.is_through = true;
        let label_entity = world
            .create_entity()
            .with(Transform::default())
            .with(rect2d)
            .with(Transparent)
            .with(Mesh2D::default())
            .with(text_render)
            .with(LayoutElement::View(View::default()))
            .with(ev_node)
            .build();
        Tree::add(world, label_entity, Some(entity));
        let raw_input = RawInput::new(label_entity);
        world.write_storage::<RawInput>().insert(entity, raw_input).unwrap();

        let mut global_events: WriteStorage<GlobalEventNode> = world.write_storage::<GlobalEventNode>();
        let mut global_event = GlobalEventNode::default();
        global_event.insert(GameEventType::RecvChar, Box::new(RawInputCallBack {entity,label:label_entity}));
        global_event.insert(GameEventType::KeyBoard, Box::new(RawInputCallBack {entity,label:label_entity}));
        global_events.insert(entity, global_event).unwrap();
    }

    pub fn update(&mut self, texts: &mut WriteStorage<TextRender>,dt:f32) {
        self.time += dt;
        if self.time >= 0.5f32 {
            self.time = 0f32;
            self.show_cursor = !self.show_cursor;
            self.update_show_cursor(self.show_cursor,texts)
        }
       
    }

    pub fn update_show_cursor(&mut self,b:bool, texts: &mut WriteStorage<TextRender>) {
        let text = texts.get_mut(self.label).unwrap();
        let ichr = if b {'|'} else { ' '};
        let up_string = string_insert_idx(self.text_value.as_str(),self.cursor_idx,ichr);
        text.set_text_string(up_string);
    }

    pub fn char_len(&self) -> usize {
        self.text_value.chars().count()
    }
}

fn string_insert_idx(str:&str,idx:usize,ichr:char) -> String {
    let mut ret = String::default();
    let mut add_idx:usize = 0;
    let mut iter = str.chars();
    while let Some(chr) = iter.next() {
        if add_idx == idx {
            ret.push(ichr)
        }
        ret.push(chr);   
        add_idx += 1;
    }
    if add_idx == idx {
        ret.push(ichr)
    }
    ret
}

fn string_rm_idx(str:&str,idx:usize) -> String {
    let mut ret = String::default();
    let mut add_idx:usize = 0;
    let mut iter = str.chars();
    while let Some(chr) = iter.next() {
        if add_idx != (idx - 1) {
            ret.push(chr);
        }
        
        add_idx += 1;
    }
    ret
}