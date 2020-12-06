use hibitset::{BitSet};
use specs::{Entity,Entities,Write,ReadStorage,Read,System,Join};

use nalgebra::{Point3,Vector3};
use crate::render::{Camera,ActiveCamera,Transparent};
use crate::common::{Transform,Hidden,HiddenPropagate};
use std::cmp::Ordering;
#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

#[derive(Default, Debug)]
pub struct SpriteVisibility {
    pub visible_unordered: BitSet,
    pub visible_ordered: Vec<Entity>,
}


#[derive(Default,Debug)]
pub struct SpriteVisibilitySortingSystem {
    centroids: Vec<Internals>,
    transparent: Vec<Internals>,
}

#[derive(Debug, Clone)]
struct Internals {
    entity: Entity,
    transparent: bool,
    centroid: Point3<f32>,
    camera_distance: f32,
    from_camera: Vector3<f32>,
}

impl SpriteVisibilitySortingSystem {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a> System<'a> for SpriteVisibilitySortingSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, SpriteVisibility>,
        ReadStorage<'a, Hidden>,
        ReadStorage<'a, HiddenPropagate>,
        Read<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Transparent>,
        ReadStorage<'a, Transform>,
    );

    fn run(&mut self,(entities, mut visibility, hidden, hidden_prop, active, camera, transparent,transform): Self::SystemData) {
       #[cfg(feature = "profiler")]
       profile_scope!("SpriteVisibilitySortingSystem");
       //todo high cpu
       let origin = Point3::<f32>::origin();
       let camera_trans = active.entity.and_then(|a| transform.get(a))
                                       .or_else(|| (&camera, &transform).join().map(|ct| ct.1).next());
       let camera_backward = camera_trans.map(|c| c.global_matrix().column(2).xyz())
                                         .unwrap_or_else(Vector3::z);
       let camera_centroid = camera_trans.map(|t| t.global_matrix().transform_point(&origin))
                                         .unwrap_or_else(|| origin);
       self.centroids.clear();
       self.centroids.extend((&*entities,&transform,!&hidden,!&hidden_prop).join().map(|(e,t,_,_)| (e,t.global_matrix().transform_point(&origin)))
                                                           .filter(|(_,c)| (c - camera_centroid).dot(&camera_backward) >= 0.0)
                                                           .map(|(entity,centroid)| Internals {
                                                               entity,
                                                               transparent:transparent.contains(entity),
                                                               centroid,
                                                               camera_distance:(centroid.z - camera_centroid.z).abs(),
                                                               from_camera: centroid - camera_centroid,
                                                           })
                           );
       visibility.visible_unordered.clear();
       visibility.visible_unordered.extend(self.centroids.iter()
                                                         .filter(|c| !c.transparent)
                                                         .map(|c| c.entity.id()));
       self.transparent.clear();
       self.transparent.extend(self.centroids.drain(..).filter(|c| c.transparent));

       self.transparent.sort_by(|a, b| {
            b.camera_distance.partial_cmp(&a.camera_distance).unwrap_or(Ordering::Equal)
        });

        visibility.visible_ordered.clear();
        visibility.visible_ordered.extend(self.transparent.iter().map(|c| c.entity));
    }


}
