#[derive(PartialEq,Clone,Default)]
pub struct Thickness {
   pub left:f64,
   pub top:f64,
   pub right:f64,
   pub bottom:f64
}

impl Thickness {
    pub fn new(l:f64,t:f64,r:f64,b:f64) -> Thickness {
        Thickness {
            left:l,
            top:t,
            right:r,
            bottom:b
        }
    }
    
    pub fn new1(num:f64) -> Thickness {
        Thickness {
            left:num,
            top:num,
            right:num,
            bottom:num
        }
    }

    pub fn new2(h:f64,v:f64) -> Thickness {
        Thickness {
            left:h,
            top:v,
            right:h,
            bottom:v
        }
    }
    
    pub fn horizontal(&self) -> f64 {
        self.left + self.right
    }

    pub fn vertical(&self) -> f64 {
        self.top + self.bottom
    }

    pub fn is_empty(&self) -> bool {
        self.left == 0f64 && self.top == 0f64 && self.right == 0f64 && self.bottom == 0f64
    }
}


#[derive(PartialEq,Eq,Clone,Copy,Debug)]
pub enum LayoutAlignment {
    Start = 0,
	Center = 1,
	End = 2,
	Fill = 3
}


impl Default for LayoutAlignment {
    fn default() -> Self { LayoutAlignment::Fill }
}

impl Into<u32> for LayoutAlignment {
    fn into(self) -> u32 { self as u32 }
}

impl From<u32> for LayoutAlignment {
    fn from(n: u32) -> LayoutAlignment {
        match n {
            0 => LayoutAlignment::Start,
            1 => LayoutAlignment::Center,
            2 => LayoutAlignment::End,
            _ => LayoutAlignment::Fill
        }
    }
}

impl LayoutAlignment {
    pub fn u32(self) -> u32 {self as u32 }
}

pub enum LNumber {
    Const(f32),
    Rate(f32)
}