// use bevy::render::mesh::{self, PrimitiveTopology, Indices};
// use bevy::math::*;      // Affine3A
// use bevy::prelude::*;   // Commands, meshes
// use walkdir::WalkDir;



// #[derive(Component, Debug)]
// pub struct  Treedata{
//     mesh:  bevy::prelude::Mesh,
//     mesh_handle: Handle<Mesh>,
//     // material: bevy_pbr::StandardMaterial>,
// }

// impl Treebuilder{

//     pub fn new() -> Self
//     {
//             Self { mesh: bevy::prelude::Mesh::new(PrimitiveTopology::TriangleList), mesh_handle: Default::default() }
//     }

//     pub fn mesh(
//         &mut self,
//     ) {
//     // Mesh Transmutation Experiment Spawning ///////////////////////////////////////////////////////
    
//         // Vertices
//         // Plain Data to 4 Triangles, facing back and forth for top and down
//         let mut ground_vertices: [[f32; 3]; 12] =   [   [-1., 0., 0.], [0.,  1., 0.], [1., 0., 0.],
//                                                     [-1., 0., 0.], [0., -1., 0.], [1., 0., 0.],
//                                                     [-1., 0., 0.], [0.,  1., 0.], [1., 0., 0.], 
//                                                     [-1., 0., 0.], [0., -1., 0.], [1., 0., 0.] ];
    
//        // Indices, not yet applied, see below
//         let ground_indices = [0, 2, 1, 3, 4, 5, 6, 7, 8, 9, 11, 10];
//         //println!("Type: {}", ground_indices.type_name());
    
//         // Transformations Matrix
//         let tmat:Affine3A = Affine3A::from_translation(Vec3{x:0.0,y:1.0,z:0.0}.into());
//         println!("T_Mat: {}, {}", tmat, tmat.translation);
    
//         //self.mesh = Mesh::new(PrimitiveTopology::TriangleList);
        
//         let mut vertexvec: Vec<[f32; 3]> = vec![];
//         let mut indexvec: Vec<u32> = vec![];
    
//         let mut cnt = 0.0; 
//         let mut cntOld = 0.0;
//         let mut cntVert = 0.0;
//         let mut rot = 0.0;
    
//         let mut countVertices = 0;
    
//         let mut add_indi: u32 = 0;
    
//         for entry in WalkDir::new("/").into_iter().filter_map(|e| e.ok()) {
//             countVertices += 12;
//             if entry.file_type().is_dir() 
//             {
//                 // Dive Up and to the side, depending on Directory
//                 for each in ground_vertices {
                    
//                     vertexvec.push(   ( // Rotation * Translation -> Transform TriangleVertex -> into Vec<[f32; 3]>
//                                         Affine3A::from_quat(Quat::from_rotation_y(rot)) *
//                                         Affine3A::from_translation(Vec3{x:cntVert*0.02,y:cnt*0.02,z:0.0})
//                                       )
//                                       .transform_point3( Vec3::from_array(each) ) 
//                                       .into()
//                 );
//                 }

//                 // dublicate indizes
//                 add_indi = 12 * (cnt as u32);
//                 indexvec.extend(vec![  0+add_indi, 2+add_indi, 1+add_indi, 
//                                              3+add_indi, 4+add_indi, 5+add_indi, 
//                                              6+add_indi, 7+add_indi, 8+add_indi, 
//                                              9+add_indi, 11+add_indi, 10+add_indi]); 

//                 // println!("Dir: {:?}", entry.path());
//                 cnt += 1.0;
//             }
//             else if entry.file_type().is_file() 
//             {
//                 cntVert += 1.0; 
//                 //println!("Filename: {:?} \n {:?} \n", entry.file_name(), entry.metadata());
    
//                 if cnt != cntOld 
//                 {
//                     rot=rot+1.0;
//                     cntVert *= -1.0;
//                     cntOld = cnt; 
//                 }
//             }
//             else if entry.file_type().is_symlink()
//             {
//                 // println!("Symlink: {:?}", entry.file_name());
//             }
//             else 
//             {
//                 //println!("Not Dir nor File: {:?}", entry.file_name());
//             }
//         }
    
//         println!("Verti: {}", countVertices);
    
//         // let mut mesh_vec = vec![];
//         // mesh_vec.extend(vec![[-1., 0., 0.], [0., 1., 0.], [1., 0., 0.] ]);
//         // mesh_vec.extend(vec![[-1., 0., 0.], [0., -1., 0.], [1., 0., 0.]]);
//         // mesh_vec.extend(vec![[-1., 0., 0.], [0., 1., 0.], [1., 0., 0.] ]);
//         // mesh_vec.extend(vec![[-1., 0., 0.], [0., -1., 0.], [1., 0., 0.]]);
    
//         self.mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; vertexvec.len()]);
//         self.mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; vertexvec.len()]);
        
//         println!("vertexvecLen: {}", vertexvec.len());
//         println!("indexvecLen: {}", indexvec.len());
    
//         &self.mesh.insert_attribute(
//             Mesh::ATTRIBUTE_POSITION,
//             vertexvec,
//         );
    
//         // In this example, normals and UVs don't matter,
//         // so we just use the same value for all of them
//         // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; 12]);
//         // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; 12]);
        
//         // // A triangle using vertices 0, 2, and 1.
//         // // Note: order matters. [0, 1, 2] will be flipped upside down, and you won't see it from behind!
//         // mesh.set_indices(Some(mesh::Indices::U32(vec![0, 2, 1, 3, 4, 5, 6, 7, 8, 9, 11, 10]))); 
//                                                     //12, 14, 13, 15, 16, 17, 18, 19, 20, 21, 23, 22
//                                                     //24, 26, 25, 27, 28, 29, 30, 32, 32, 33, 35, 34
    
//         &self.mesh.set_indices(Some(mesh::Indices::U32(indexvec)));
// //        let cube_mesh_handle: Handle<Mesh> = Mesh2dHandle.add(mesh);
//     }

//     pub fn mesh_handle (&mut self, mes_que: &mut ResMut<Assets<Mesh>>)
//     {
//         self.mesh_handle = mes_que.add(self.mesh);
//     }

//     pub fn get_mesh_handle (&self) -> Handle<Mesh>
//     {
//         return self.mesh_handle;
//     }

// }
