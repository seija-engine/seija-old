mod window;
pub use window::{WindowModule};

pub struct ViewPortSize {
    width:f64,
    height:f64
}

impl ViewPortSize {
    pub fn new(w:f64,h:f64) -> Self {
        ViewPortSize {
            width:w,
            height:h
        }
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }
}