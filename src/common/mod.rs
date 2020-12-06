pub mod transform;
pub mod rect;
pub mod tree;
mod update;
pub use update::{Update,UpdateDesc,UpdateSystem,UpdateType,UpdateCallBack};
pub use rect::{Rect2D};
pub use transform::transform::{Transform};
pub use transform::component::{TransformSystem};
pub use tree::{Tree,TreeNode,TreeEvent};
use specs::{Component,NullStorage,FlaggedStorage,DenseVecStorage};

#[derive(Default)]
pub struct EntityInfo {
    pub name:String,
    pub tag:u32,
    pub layer:u32
}

impl Component for EntityInfo {
    type Storage = FlaggedStorage<Self, DenseVecStorage<Self>>;
}

#[derive(Clone, Debug, Default)]
pub struct Hidden;

impl Component for Hidden {
    type Storage = NullStorage<Self>;
}

#[derive(Clone, Debug, Default)]
pub struct HiddenPropagate;

impl Component for HiddenPropagate {
    type Storage = FlaggedStorage<Self, NullStorage<Self>>;
}

#[derive(Copy,Clone,Eq, PartialEq)]
pub enum Horizontal {
    Left,
    Center,
    Right
}

#[derive(Copy,Clone,Eq, PartialEq)]
pub enum Vertical {
    Top,
    Center,
    Bottom
}

#[derive(Copy,Clone,Eq, PartialEq)]
pub enum AnchorAlign {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight
}

impl From<u32> for AnchorAlign {
    fn from(num: u32) -> AnchorAlign {
        match num {
            0 => AnchorAlign::TopLeft,
            1 => AnchorAlign::Top,
            2 => AnchorAlign::TopRight,
            3 => AnchorAlign::Left,
            4 => AnchorAlign::Center,
            5 => AnchorAlign::Right,
            6 => AnchorAlign::BottomLeft,
            7 => AnchorAlign::Bottom,
            8 => AnchorAlign::BottomRight,
            _ => AnchorAlign::Center
        }
    }
}

impl AnchorAlign {
    pub fn to_hv_align(&self) -> (Horizontal,Vertical) {
        match self {
            AnchorAlign::TopLeft => (Horizontal::Left,Vertical::Top),
            AnchorAlign::Top => (Horizontal::Center,Vertical::Top),
            AnchorAlign::TopRight => (Horizontal::Right,Vertical::Top),
            AnchorAlign::Left => (Horizontal::Left,Vertical::Center),
            AnchorAlign::Center => (Horizontal::Center,Vertical::Center),
            AnchorAlign::Right => (Horizontal::Right,Vertical::Center),
            AnchorAlign::BottomLeft => (Horizontal::Left,Vertical::Center),
            AnchorAlign::Bottom => (Horizontal::Center,Vertical::Bottom),
            AnchorAlign::BottomRight => (Horizontal::Right,Vertical::Bottom),
        }
    }
}