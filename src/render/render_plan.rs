use rendy::graph::{NodeId,ImageId};
use std::collections::{HashMap};

pub struct RenderPlan {
    pub node_passes:HashMap<String,NodeId>,
    pub outputs:HashMap<String,ImageId>,
    pub depths:HashMap<String,ImageId>,
}

impl RenderPlan {
    pub fn new() -> Self {
        RenderPlan {
            node_passes: HashMap::new(),
            outputs: HashMap::new(),
            depths:  HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.node_passes.clear();
        self.outputs.clear();
        self.depths.clear();
    }
}