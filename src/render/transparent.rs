use specs::{Component,NullStorage};

#[derive(Clone, Copy, Debug, Default)]
pub struct Transparent;

impl Component for Transparent {
    type Storage = NullStorage<Self>;
}