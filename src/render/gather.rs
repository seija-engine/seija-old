use crate::render::pod::{ViewArgs,IntoPod};
use crate::render::{Camera,ActiveCamera};
use crate::common::{Transform};
use glsl_layout::*;
use nalgebra::{convert,Vector3,Matrix4};
use specs::{World,Read,ReadStorage,SystemData,Join};

type Std140<T> = <T as AsStd140>::Std140;

#[derive(Debug)]
pub struct CameraGatherer {
    pub camera_position: vec3,
    pub projview: Std140<ViewArgs>
}

impl CameraGatherer {
    pub fn gather(world: &World) -> Self {
       
        let (active_camera,cameras,transforms) = <(Read<'_, ActiveCamera>, 
                                                   ReadStorage<'_, Camera>, 
                                                   ReadStorage<'_, Transform>)>::fetch(world);
        
        let identity = Transform::default();
        let defcam = Camera::standard_2d(1.0, 1.0);
        let (camera, transform) = active_camera.entity.as_ref().and_then(|ac| {
            cameras.get(*ac).map(|camera| (camera, transforms.get(*ac).unwrap_or(&identity)))
        }).unwrap_or_else(|| {
            (&cameras, &transforms).join().next().unwrap_or((&defcam, &identity))
        });
        let camera_position =  convert::<_, Vector3<f32>>(transform.global_matrix().column(3).xyz()).into_pod();
        let proj = camera.as_matrix();
        let view = transform.global_view_matrix();

        let proj_view: [[f32; 4]; 4] = ((*proj) * view).into();
        let proj: [[f32; 4]; 4] = (*proj).into();
        let view: [[f32; 4]; 4] = convert::<_, Matrix4<f32>>(transform.global_view_matrix()).into();
        let projview = ViewArgs {
            proj: proj.into(),
            view: view.into(),
            proj_view: proj_view.into(),
        }.std140();
        Self {
            camera_position,
            projview,
        }
    }
}