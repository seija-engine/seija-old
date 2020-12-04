#[derive(Clone)]
pub enum LayoutType {
    View,
    Layout,
    StackLayout
}
#[derive(Clone)]
pub struct LayoutHandle {
    eid:u32,
    typ:LayoutType
}

impl LayoutHandle {
    pub fn new(eid:u32,typ:LayoutType) -> LayoutHandle {
        LayoutHandle {
            eid,
            typ
        }
    } 
    pub fn eid(&self) -> u32 {
        self.eid
    }
    pub fn typ(&self) -> &LayoutType {
        &self.typ
    }
}