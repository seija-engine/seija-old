use rendy::graph::render::{RenderGroupBuilder};
use rendy::hal::Backend;
use specs::{World};
use crate::render::{OutputOptions};
#[derive(Debug)]
pub struct GraphNode<B:Backend> {
    node_name:String,
    pub groups: Vec<(i32,Box<dyn RenderGroupBuilder<B,World>>)>,
    pub outputs: OutputOptions<B>,
    pub deps:Vec<String>
}

impl<B> GraphNode<B> where B:Backend {
    pub fn add_dep(&mut self,node_name:String) {
        self.deps.push(node_name);
    }

    pub fn node_name(&self) -> String {
        self.node_name.clone()
    }

    pub fn deps(&self) -> &Vec<String> {
        &self.deps
    }
}

pub struct GraphNodeBuilder<B:Backend> {
    node_name:String,
    groups: Vec<(i32,Box<dyn RenderGroupBuilder<B,World>>)>,
    deps:Vec<String>
}

impl<B> Default for GraphNodeBuilder<B> where B:Backend {
    fn default() -> Self {
        GraphNodeBuilder {
            node_name: String::from(""),
            groups:  Vec::new(),
            deps:Vec::new()
        }
    }
}

impl<B>  GraphNodeBuilder<B> where B:Backend {
    pub fn new() -> Self {
        GraphNodeBuilder::default()
    }
    pub fn with_name(mut self,name:&str) -> Self {
        self.node_name = String::from(name);
        self
    }


    pub fn with_dep(mut self,name:&str) -> Self {
        self.deps.push(String::from(name));
        self
    }

    pub fn with_group(mut self,order_id: impl Into<i32>,group:impl RenderGroupBuilder<B,World> + 'static) -> Self {
        let boxed_group = Box::new(group);
        self.groups.push((order_id.into(),boxed_group));
        self
    }

    pub fn build(self,output:OutputOptions<B>) -> GraphNode<B> {
        GraphNode {
            node_name: self.node_name,
            groups: self.groups,
            outputs: output,
            deps: self.deps
        }
    }
}