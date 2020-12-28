use specs::{storage::{DenseVecStorage},Component};
use crate::assets::{Handle};
use crate::render::{FontAsset};
use crate::common::{AnchorAlign};

#[derive(Debug, Clone, Eq, PartialEq,Copy)]
pub enum LineMode {
    Single,
    Wrap,
}

impl From<u32> for LineMode {
    fn from(u: u32) -> Self {
        match u {
            0 => LineMode::Single,
            _ => LineMode::Wrap
        }
    }
}

pub struct TextRender {
    pub text:String,
    pub font_size:i32,
    pub color:[f32;4],
    pub font:Option<Handle<FontAsset>>,
    pub line_mode:LineMode,
    pub anchor:AnchorAlign,
    pub auto_size:bool
}

impl Component for TextRender {
    type Storage = DenseVecStorage<Self>;
}

impl  TextRender {
    pub fn new(font:Option<Handle<FontAsset>>) -> TextRender {
        TextRender {
            text : String::from(""),
            font_size: 16,
            color: [0f32,0f32,0f32,1f32],
            font,
            line_mode:LineMode::Single,
            anchor:AnchorAlign::Center,
            auto_size:false
        }
    }

    pub fn set_text(&mut self,txt:&str) {
        self.text = String::from(txt);
    }


    pub fn set_font_size(&mut self,font_size:i32) {
        self.font_size = font_size;
    }

    pub fn is_valid(&self) -> bool {
       self.font.is_some()
    }


    pub fn set_color(&mut self,r:f32,g:f32,b:f32,a:f32) {
        self.color[0] = r;
        self.color[1] = g;
        self.color[2] = b;
        self.color[3] = a;
    }

    pub fn set_line_mode(&mut self,line_model:LineMode) {
        self.line_mode = line_model;
    }

    pub fn line_mode(&self) -> LineMode {
        self.line_mode
    }

    pub fn set_anchor(&mut self,anchor:AnchorAlign) {
        self.anchor = anchor
    }

    pub fn anchor(&self) -> AnchorAlign {
        self.anchor
    }
}
