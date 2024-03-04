use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::math::*;      // Affine3A
use bevy::prelude::*;   
use walkdir::WalkDir;

use meshtext::{MeshGenerator, IndexedMeshText, TextSection};


use crate::treedata::Treedata;

#[derive(Component, Debug)]
pub struct  Treebuilder{
    mesh:  bevy::prelude::Mesh,
    // mesh_handle: Handle<Mesh>,
    // material: bevy_pbr::StandardMaterial>,
}

impl Treebuilder {
    pub fn new() -> Self
    {
            Self { mesh: bevy::prelude::Mesh::new(PrimitiveTopology::TriangleList) }
    }


    // Lässt sich ziemlich sicher umschreiben als Subfunktion der Setupfunktion, welche das mesh direkt an die Richtige Stelle 
    // in meshes speichert und beim "spwanen" mit dem entsprechendem Marker versieht. Von dort lässt sich alles einzelne
    // über "attribute_mut" aufgreifen und in subfunktionen anpassen. Hopefully
    pub fn generate_mesh(
        &mut self,
        mut meshTreedata: &mut ResMut<Treedata>,
        // commands: &mut Commands,
        // mut meshes: ResMut<Assets<Mesh>>,
        // mesh:  bevy::prelude::Mesh,
    ) -> Mesh {
        
    // Mesh Transmutation Experiment Spawning ///////////////////////////////////////////////////////
    
        // Vertices
        // Plain Data to 4 Triangles, facing back and forth for top and down
        let mut ground_vertices: [[f32; 3]; 12] =   [   [-1., 0., 0.], [0.,  1., 0.], [1., 0., 0.],
                                                        [-1., 0., 0.], [0., -1., 0.], [1., 0., 0.],
                                                        [-1., 0., 0.], [0.,  1., 0.], [1., 0., 0.], 
                                                        [-1., 0., 0.], [0., -1., 0.], [1., 0., 0.] ];
    
       // Indices, not yet applied, see below
        let ground_indices = [0, 2, 1, 3, 4, 5, 6, 7, 8, 9, 11, 10];
        //println!("Type: {}", ground_indices.type_name());
    
        // Transformations Matrix
        let tmat:Affine3A = Affine3A::from_translation(Vec3{x:0.0,y:1.0,z:0.0}.into());
        println!("T_Mat: {}, {}", tmat, tmat.translation);
        
        let mut vertexvec: Vec<[f32; 3]> = vec![];
        let mut indexvec: Vec<u32> = vec![];
    
        let mut cnt = 0.0; 
        let mut cntOld = 0.0;
        let mut cntVert = 0.0;
        let mut rot = 0.0;
    
        let mut countVertices = 0;
    
        let mut add_indi: u32 = 0;
    
        for entry in WalkDir::new("/home").into_iter().filter_map(|e| e.ok()) {
            countVertices += 12;
            if entry.file_type().is_dir() 
            {

                    // // Text to describe the controls.
                    // commands.spawn(
                    //     TextBundle::from_section(
                    //         entry.file_name().to_str().unwrap(),
                    //         TextStyle {
                    //             font_size: 20.0,
                    //             ..default()
                    //         },
                    //     )
                    //     .with_style(Style {
                    //         position_type: PositionType::Absolute,
                    //         top: Val::Px(12.0),
                    //         left: Val::Px(12.0),
                    //         ..default()
                    //     }),
                    // );



                // Dive Up and to the side, depending on Directory 
                for each in ground_vertices { 
                    
                    vertexvec.push(   ( // Rotation * Translation -> Transform TriangleVertex -> into Vec<[f32; 3]>
                                        Affine3A::from_quat(Quat::from_rotation_y(rot)) *
                                        Affine3A::from_translation(Vec3{x:cntVert*0.02,y:cnt*0.02,z:0.0})
                                      )
                                      .transform_point3( Vec3::from_array(each) ) 
                                      .into()
                );
                }

                // dublicate indizes
                add_indi = 12 * (cnt as u32);
                indexvec.extend(vec![  0+add_indi, 2+add_indi, 1+add_indi, 
                                             3+add_indi, 4+add_indi, 5+add_indi, 
                                             6+add_indi, 7+add_indi, 8+add_indi, 
                                             9+add_indi, 11+add_indi, 10+add_indi]); 

                // println!("Dir: {:?}", entry.path());
                // print!("Dir: {:?} Depth: {:?}", entry.path(), entry.depth() );
                // println!();

                cnt += 1.0;
            }
            else if entry.file_type().is_file() 
            {
                cntVert += 1.0; 
                // println!("Filename: {:?} \n {:?} \n", entry.file_name(), entry.metadata());
    
                if cnt != cntOld 
                {
                    rot      = rot+1.0;
                    cntVert *= -1.0;
                    cntOld   = cnt; 
                }
            }
            else if entry.file_type().is_symlink()
            {
                // println!("Symlink: {:?}", entry.file_name());
            }
            else 
            {
                //println!("Not Dir nor File: {:?}", entry.file_name());
            }
        }
    
        println!("Verti: {}", countVertices);
    
        // let mut mesh_vec = vec![];
        // mesh_vec.extend(vec![[-1., 0., 0.], [0., 1., 0.], [1., 0., 0.] ]);
        // mesh_vec.extend(vec![[-1., 0., 0.], [0., -1., 0.], [1., 0., 0.]]);
        // mesh_vec.extend(vec![[-1., 0., 0.], [0., 1., 0.], [1., 0., 0.] ]);
        // mesh_vec.extend(vec![[-1., 0., 0.], [0., -1., 0.], [1., 0., 0.]]);
        
        meshTreedata.mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; vertexvec.len()]);
        meshTreedata.mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; vertexvec.len()]);
        
        println!("vertexvecLen: {}", vertexvec.len());
        println!("indexvecLen: {}", indexvec.len());
    
        meshTreedata.mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vertexvec,
        );
    
        // In this example, normals and UVs don't matter,
        // so we just use the same value for all of them
        // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; 12]);
        // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; 12]);
        
        // // A triangle using vertices 0, 2, and 1.
        // // Note: order matters. [0, 1, 2] will be flipped upside down, and you won't see it from behind!
        // mesh.set_indices(Some(mesh::Indices::U32(vec![0, 2, 1, 3, 4, 5, 6, 7, 8, 9, 11, 10]))); 
                                                    //12, 14, 13, 15, 16, 17, 18, 19, 20, 21, 23, 22
                                                    //24, 26, 25, 27, 28, 29, 30, 32, 32, 33, 35, 34

        meshTreedata.mesh.set_indices(Some(mesh::Indices::U32(indexvec)));

        // Handing back the generated mesh
        meshTreedata.mesh.clone()

    }

    // pub fn generate_mesh_dodeca(
    //     &mut self,
    //     mut meshTreedata: &mut ResMut<Treedata>,

    // ) -> Mesh {

    //     // Vertices
    //     // Plain Data to 4 Triangles, facing back and forth for top and down
    //     let mut ground_vertices: [[f32; 3]; 12] = 


    // }

    // Tests of walkdir, iterating over ground level to count size of branches 
    pub fn dirwalk(&self) {
        for entry in WalkDir::new("/").max_depth(1).into_iter().filter_map(|e| e.ok()) {
            println!("Dir: {:?}", entry.path());
            for entry2 in WalkDir::new(entry.path()).min_depth(1).max_depth(2).into_iter().filter_map(|e| e.ok()) {
                if entry2.file_type().is_file() 
                {
                    print!("File: {:?} ", entry2.path());
                    println!();
                }
                if entry2.file_type().is_dir()
                {
                    // print!("SubDir: {:?} \t {:?} ", entry.path(), entry.metadata());
                    print!("SubDir: {:?} Depth: {:?}", entry2.path(), entry2.depth() );
                    println!();
                    // println!();
                }
            }
            println!();
            println!();
        }
    }




// =============================================================== //


    pub fn generate_text_mesh(
        &mut self,
        mut meshTreedata: &mut ResMut<Treedata>,
    ) -> Mesh {

        let font_data = include_bytes!("/home/nero/code/rust/storytree/assets/fonts/Roboto-Thin.ttf");
        let mut generator = MeshGenerator::new(font_data);
    
        // indices.extend(vec![ 1, 2, 3, 4 ]);
    
        // println!("VertCount: {:?}", vertices.len());
        // println!("InCount: {:?}", indices.len());
    
        // let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
        // // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; positions.len()]);
        // mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    
        // mesh.compute_flat_normals();
    
        // mesh.set_indices(Some(mesh::Indices::U32(indices)));
    

        // Vertices
        // Plain Data to 4 Triangles, facing back and forth for top and down
        let mut ground_vertices: [[f32; 3]; 12] =   [   [-1., 0., 0.], [0.,  1., 0.], [1., 0., 0.],
                                                    [-1., 0., 0.], [0., -1., 0.], [1., 0., 0.],
                                                    [-1., 0., 0.], [0.,  1., 0.], [1., 0., 0.], 
                                                    [-1., 0., 0.], [0., -1., 0.], [1., 0., 0.] ];
    
       // Indices, not yet applied, see below
        let ground_indices = [0, 2, 1, 3, 4, 5, 6, 7, 8, 9, 11, 10];
        //println!("Type: {}", ground_indices.type_name());
    
        // Transformations Matrix
        let tmat:Affine3A = Affine3A::from_translation(Vec3{x:0.0,y:1.0,z:0.0}.into());
        println!("T_Mat: {}, {}", tmat, tmat.translation);

        let mut vertexvec: Vec<[f32; 3]> = vec![];
        let mut indexvec: Vec<u32> = vec![];
    
        let mut cnt = 0.0; 
        let mut cntOld = 0.0;
        let mut cntVert = 0.0;
        let mut rot = 0.0;
    
        let mut countVertices = 0;
    
        let mut add_indi: u32 = 0;

        for entry in WalkDir::new("/home/ben/projects/rust/storytree/").into_iter().filter_map(|e| e.ok()) {
            countVertices += 12;
            if entry.file_type().is_dir() 
            {
                // Adjust position
                let mut transform: [f32; 16] =       [1.0, 0.0, 0.0, 0.0,
                                                        0.0, 1.0, 0.0, 0.0,
                                                        0.0, 0.0, 1.0, 0.0,
                                                        0.0, 0.0, 0.0, 1.0];
                //  = ( Affine3A::from_quat(Quat::from_rotation_y(rot)) *
                // Affine3A::from_translation(Vec3{x:cntVert*0.02,y:cnt*0.02,z:0.0}) ).to_cols_array_2d();

                // let affine_transform = Affine3A::from_quat(Quat::from_rotation_y(rot)) * Affine3A::from_translation(Vec3{x:cntVert*0.02,y:cnt*0.02,z:0.0});
                // affine_transform.write_cols_to_slice(&mut transform[0..12]);

                let result: IndexedMeshText = generator
                .generate_section(
                entry.file_name().to_str().unwrap(),
                false,
                Some(&transform)
                )
                .expect("Failed to generate mesh.");

                let vertices = result.vertices;
                let positions: Vec<[f32; 3]> = vertices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
                // let uvs = vec![[0f32, 0f32]; positions.len()];    

                let indices = result.indices;

                println!("positions: {}", positions.len());
                println!("indices: {}", indices.len());
                println!();

                vertexvec.extend(positions);
                indexvec.extend(indices);

                // Dive Up and to the side, depending on Directory 
                // for each in ground_vertices { 
                    
                //     vertexvec.push(   ( // Rotation * Translation -> Transform TriangleVertex -> into Vec<[f32; 3]>
                //                         Affine3A::from_quat(Quat::from_rotation_y(rot)) *
                //                         Affine3A::from_translation(Vec3{x:cntVert*0.02,y:cnt*0.02,z:0.0})
                //                       )
                //                       .transform_point3( Vec3::from_array(each) ) 
                //                       .into()
                // );
                // }

                // // dublicate indizes
                // add_indi = 12 * (cnt as u32);
                // indexvec.extend(vec![  0+add_indi, 2+add_indi, 1+add_indi, 
                //                              3+add_indi, 4+add_indi, 5+add_indi, 
                //                              6+add_indi, 7+add_indi, 8+add_indi, 
                //                              9+add_indi, 11+add_indi, 10+add_indi]); 

                // println!("Dir: {:?}", entry.path());
                // print!("Dir: {:?} Depth: {:?}", entry.path(), entry.depth() );
                // println!();

                cnt += 1.0;
            }
            else if entry.file_type().is_file() 
            {
                cntVert += 1.0; 
                // println!("Filename: {:?} \n {:?} \n", entry.file_name(), entry.metadata());
    
                if cnt != cntOld 
                {
                    rot      = rot+1.0;
                    cntVert *= -1.0;
                    cntOld   = cnt; 
                }
            }
            else if entry.file_type().is_symlink()
            {
                // println!("Symlink: {:?}", entry.file_name());
            }
            else 
            {
                //println!("Not Dir nor File: {:?}", entry.file_name());
            }
        }
    
        println!("Verti: {}", countVertices);
    
        // let mut mesh_vec = vec![];
        // mesh_vec.extend(vec![[-1., 0., 0.], [0., 1., 0.], [1., 0., 0.] ]);
        // mesh_vec.extend(vec![[-1., 0., 0.], [0., -1., 0.], [1., 0., 0.]]);
        // mesh_vec.extend(vec![[-1., 0., 0.], [0., 1., 0.], [1., 0., 0.] ]);
        // mesh_vec.extend(vec![[-1., 0., 0.], [0., -1., 0.], [1., 0., 0.]]);
        
        meshTreedata.mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; vertexvec.len()]);
        meshTreedata.mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; vertexvec.len()]);
        
        println!("vertexvecLen: {}", vertexvec.len());
        println!("indexvecLen: {}", indexvec.len());
    
        meshTreedata.mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vertexvec,
        );
    
        // In this example, normals and UVs don't matter,
        // so we just use the same value for all of them
        // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; 12]);
        // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; 12]);
        
        // // A triangle using vertices 0, 2, and 1.
        // // Note: order matters. [0, 1, 2] will be flipped upside down, and you won't see it from behind!
        // mesh.set_indices(Some(mesh::Indices::U32(vec![0, 2, 1, 3, 4, 5, 6, 7, 8, 9, 11, 10]))); 
                                                    //12, 14, 13, 15, 16, 17, 18, 19, 20, 21, 23, 22
                                                    //24, 26, 25, 27, 28, 29, 30, 32, 32, 33, 35, 34

        meshTreedata.mesh.set_indices(Some(mesh::Indices::U32(indexvec)));

        meshTreedata.mesh.clone()

    }

}

