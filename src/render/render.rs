use crate::render::graph_node::{GraphNode};
use rendy::hal::{Instance as _};
use rendy::graph::{GraphBuilder,Graph};
use rendy::command::{Families,QueueId};
use rendy::core::{Instance};
use rendy::graph::render::{RenderPassNodeBuilder,SubpassBuilder};
use std::collections::{HashMap};
use crate::render::render_plan::{RenderPlan};
use specs::{World,WorldExt,};
use rendy::factory::{Factory};
use crate::render::{OutputColor,Camera,Transparent,env::{FontEnv}};
use rendy::core::hal::window::{Extent2D};
use crate::render::components::{ImageRender,SpriteRender,TextRender,Mesh2D};
use crate::assets::{AssetStorage};
use crate::render::types::{Backend,Texture};


pub struct RenderSystem<B:Backend> {
    families:Families<B>,
    graph:Option<Graph<B,World>>,
    render_builder:Option<RenderBuilder<B>>,
}

impl<B> RenderSystem<B> where B:Backend {
    pub fn new(world:&mut World) -> Self {
        let config: rendy::factory::Config = Default::default();
        let instance = B::Instance::create("Rendy", 1).unwrap();
        let (factory,families) = rendy::factory::init_with_instance(Instance::new(instance), &config).unwrap();
        let queue_id = QueueId {
            family: families.family_by_index(0).id(),
            index: 0,
        };
        let plan = RenderPlan::new();
        world.insert(factory);
        world.insert(plan);
        world.insert(queue_id);
        world.register::<Camera>();
        world.register::<ImageRender>();
        world.register::<SpriteRender>();
        world.register::<Transparent>();
        world.register::<TextRender>();
        world.register::<Mesh2D>();
        world.insert(FontEnv::<B>::default());
        
        RenderSystem {
            families: families,
            graph: None,
            render_builder:None,
        }
    }


    pub fn build(&mut self,mut builder:RenderBuilder<B>,world:&World) {
        let graph_builder = builder.build(&mut world.fetch_mut::<RenderPlan>());
        let graph = graph_builder.build(&mut world.fetch_mut::<Factory<B>>(),&mut self.families,&world);
        self.graph = Some(graph.unwrap());
        self.render_builder = Some(builder);
    }

    pub fn re_build(&mut self,world:&World) {
        world.fetch_mut::<RenderPlan>().clear();
        let graph_builder = self.render_builder.as_mut().unwrap().build(&mut world.fetch_mut::<RenderPlan>());
        let new_graph = graph_builder.build(&mut world.fetch_mut::<Factory<B>>(),&mut self.families,world).unwrap();
        if let Some(graph) = self.graph.take() {
            graph.dispose(&mut world.fetch_mut::<Factory<B>>(), world);
        }
        self.graph = Some(new_graph);
    }

    pub fn update(&mut self,world:&World) {
        if self.graph.is_some() {
            {
                let mut factory = world.fetch_mut::<Factory<B>>();
                factory.maintain(&mut self.families);
                self.graph.as_mut().unwrap().run(&mut factory,&mut self.families,world);
            }
        }
    }

    pub fn dispose(self, world: &mut World) {
      
        //if let Some(graph) = self.graph.take() {
        //    let mut factory = world.fetch_mut::<Factory<B>>();
        //    graph.dispose(&mut *factory, world);
        //}

        if let Some(mut storage) = world.try_fetch_mut::<AssetStorage<Texture>>() {
            storage.unload_all();
        }

        drop(self.families);
        
    }
}

pub struct RenderBuilder<B:Backend> {
    nodes:HashMap<String,GraphNode<B>>,
    roots:Vec<String>,
    graph_builder:Option<GraphBuilder<B,World>>,
    render_size:Extent2D
}

impl<B> RenderBuilder<B> where B:Backend {
    pub fn new() -> Self {
        RenderBuilder {
            nodes:HashMap::new(),
            roots:Vec::new(),
            graph_builder:None,
            render_size:Extent2D {width:1024,height:768}
        }
    }
    pub fn with_node(mut self,node:GraphNode<B>) -> RenderBuilder<B> {
        self.nodes.insert(node.node_name(), node);
        self
    }

    pub fn with_root_node(mut self,node:GraphNode<B>) -> RenderBuilder<B> {
        let node_name = node.node_name();
        self.nodes.insert(node_name.clone(),node);
        self.roots.push(node_name);
        self
    }

    pub fn build(&mut self,plan:&mut RenderPlan) -> GraphBuilder<B,World> {
        self.graph_builder = Some(GraphBuilder::new());
        let mut build_lst:Vec<String> = Vec::new();
        for node_name in self.roots.iter() {
            let mut dep_names = self.get_node_dep_names(self.nodes.get(node_name));
            build_lst.append(&mut dep_names);
        }
        let mut drain_node:HashMap<String,GraphNode<B>> = self.nodes.drain().map(|s| s).collect();
        let drain_node_ref = &mut drain_node;
        for node_name in build_lst {
            let node = drain_node_ref.get_mut(&node_name).unwrap();
            self.eval_node(node,plan);
        };
        self.graph_builder.take().unwrap()
    }

    fn get_node_dep_names(&self,may_node:Option<&GraphNode<B>>) -> Vec<String> {
       let mut ret = Vec::new();
       if let Some(node) = may_node {
           for cnode in node.deps() {
            let mut dep_names = self.get_node_dep_names(self.nodes.get(cnode));
            ret.append(&mut dep_names);
           }
           ret.push(node.node_name()); 
       }
       ret
    }

    fn eval_node(&mut self,node:&mut GraphNode<B>,plan:&mut RenderPlan) {
        let mut subpass = SubpassBuilder::new();
        let mut pass = RenderPassNodeBuilder::new();
        let node_name = node.node_name();
        let graph_builder = self.graph_builder.as_mut().unwrap();
        node.groups.sort_by_key(|a|a.0);
        for group in node.groups.drain(..).map(|a|a.1) {
            subpass.add_dyn_group(group);
        }
        for (_,color) in node.outputs.colors.drain(..).enumerate() {
            match color {
                OutputColor::Image(img) => {
                   let image_id = graph_builder.create_image(img.kind,img.levels,img.format,img.clear);
                   plan.outputs.insert(node_name.clone(),image_id);
                   subpass.add_color(image_id);
                },
                OutputColor::Surface(surface, clear) => {
                    subpass.add_color_surface();
                    pass.add_surface(surface,self.render_size,clear);
                }
            }
        }
        if let Some(img) = node.outputs.depth {
            let image_id = graph_builder.create_image(img.kind,img.levels,img.format,img.clear);
            plan.depths.insert(node_name.clone(),image_id);
            subpass.set_depth_stencil(image_id);
        }
        for dep_name in node.deps.iter() {
            let node_id = plan.node_passes.get(dep_name).expect("not found nodeid");
            subpass.add_dependency(*node_id);
        }
        pass.add_subpass(subpass);
        let node_id = graph_builder.add_node(pass);
        plan.node_passes.insert(node_name.clone(),node_id);
    }
}

#[test]
fn test_build() {
    use crate::render::graph_node::{GraphNodeBuilder};
    use crate::render::OutputOptions;
    let pre_node0:GraphNode<rendy::vulkan::Backend> = GraphNodeBuilder::new()
        .with_name("pre_node0").build(OutputOptions {
        colors:Vec::new(),
        depth:None
    });
    let pre_node1:GraphNode<rendy::vulkan::Backend> = GraphNodeBuilder::new()
    .with_name("pre_node1").with_dep("pre_node0")
    .build(OutputOptions {
        colors:Vec::new(),
        depth:None
    });
    let main_node:GraphNode<rendy::vulkan::Backend> = GraphNodeBuilder::new()
    .with_name("main_node").with_dep("pre_node1")
    .build(OutputOptions {
        colors:Vec::new(),
        depth:None
    });
    let _rb = RenderBuilder::new().with_root_node(main_node).with_node(pre_node0).with_node(pre_node1);
}