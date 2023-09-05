// Treedata struct is a singleton Resource, holding the mainmesh

use bevy::prelude::*;   
use bevy::render::mesh::{PrimitiveTopology};

#[derive(Resource)]
pub struct Treedata{
    pub mesh: bevy::prelude::Mesh,
    pub mesh_handle: Handle<Mesh>,
}

impl FromWorld for Treedata {
    fn from_world(_world: &mut World) -> Self {
        // You have full access to anything in the ECS World from here.
        // For example, you can access (and mutate!) other resources:
        // let mut x = world.resource_mut::<MyOtherResource>();
        // x.do_mut_stuff();
    
        // println!( "TotalCountEntities: {}", world.entities().total_count() );

        Treedata{ mesh: Mesh::new(PrimitiveTopology::TriangleList), mesh_handle: Default::default() } 
    }
}