
use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::math::*;      // Affine3A
use bevy::prelude::*;   
use walkdir::WalkDir;


pub fn generate_space_mesh(
    //mut meshTreedata: &mut ResMut<Treedata>,
) -> Mesh {
        let PHI: f32 = 1.618033989; 

        // Vertices
        // Plain Data dodecaeder
        let mut ground_vertices: [[f32; 3]; 20] =   
            [   [  0.,      -1./PHI,  -PHI ], // 0
                [  1.,      -1.,      -1.  ], // 1
                [  1./PHI,  -PHI,     0.   ], // 2
                [  -1./PHI, -PHI,     0.   ], // 3
                [  -1.,     -1.,      -1.  ], // 4

                [  -1.,      -1.,      1.  ], // 5
                [  0.,      -1./PHI,  PHI  ], // 6
                [  1.,      -1.,      1.   ], // 7

                [  PHI,      0.,    1./PHI ], // 8
                [  PHI,      0.,   -1./PHI ], // 9
                [  -PHI,     0.,   -1./PHI ], // 10
                [  -PHI,     0.,    1./PHI ], // 11

                // Gespiegelt an der XZ Ebene, -> Werte von 0 - 7 mit positiver y achse

                [  -1.,      1.,      1.  ], // 12
                [  0.,       1./PHI,  PHI  ], // 13
                [  1.,       1.,      1.   ], // 14
                [  1.,       1.,      -1.  ], // 15
                [  0.,       1./PHI,  -PHI ], // 16
                [  -1.,      1.,      -1.  ], // 17
                [  -1./PHI,  PHI,     0.   ], // 18
                [  1./PHI,   PHI,     0.   ]  // 19 
                ];

        let ground_indices = [         
                0, 1, 2,
                0, 2, 3,
                0, 3, 4,

                6, 5, 3,
                6, 3, 2, 
                6, 2, 7,

                2, 1, 9,
                2, 9, 8, 
                2, 8, 7,

                3, 5, 11, 
                3, 11, 10,
                3, 10, 4,

                5, 6, 13, 
                5, 13, 12, 
                5, 12, 11, 

                1, 0, 16, 
                1, 16, 15, 
                1, 15, 9, 

                // Halber Dodecaeder Formuliert in symmetrie, für möglichkeit zur animierten öffnung

                7, 8, 14, 
                7, 14, 13, 
                7, 13, 6, 

                4, 10, 17, 
                4, 17, 16, 
                4, 16, 0, 

                13, 14, 19, 
                13, 19, 18, 
                13, 18, 12, 

                16, 17, 18, 
                16, 18, 19, 
                16, 19, 15, 

                18, 17, 10, 
                18, 10, 11, 
                18, 11, 12, 

                19, 14, 8, 
                19, 8, 9, 
                19, 9, 15, 
            ];

            // Transformations Matrix
            let tmat:Affine3A = Affine3A::from_translation(Vec3{x:0.0,y:1.0,z:0.0}.into());

            let mut vertexvec: Vec<[f32; 3]> = vec![];
            let mut indexvec: Vec<u32> = vec![];

            let mut cnt = 0.0; 
            let mut cntOld = 0.0;
            let mut cntVert = 0.0;
            // let mut rot = 0.0;
        
            let mut countVertices = 0;
        
            let mut add_indi: u32 = 0;


            let mut rot : Vec3 = Vec3{x:0.0,
                                      y:1.0,
                                      z:0.0};

            let mut trans : Vec3 = Vec3{x:0.0,
                                        y:1.0,
                                        z:0.0};

            let mut oldDepth = 0;

            let mut cntAllFiles = 0; // was once 4 330 659
                                     // dirs then: 430 573
                                     // Soo we print dir names to texture and bind as uv? 
                                     // and number of files holding gets represeneted how? 
                                     
            for entry in WalkDir::new("/").into_iter().filter_map(|e| e.ok()) {        
            // for entry in WalkDir::new("./TestTree").into_iter().filter_map(|e| e.ok()) {
                // println!("Entry: {:?} Depth: {:?}", entry.path(), entry.depth()  );
                

                if entry.file_type().is_dir() 
                {
                    
                countVertices += 20;


                // println!("Entry: {:?} Depth: {:?}", entry.path(), entry.depth()  );


                // Depth \
                if oldDepth < entry.depth() 
                {
                    rot.y      = rot.y+1.0;
                    trans.y = trans.y + 1.;

                    // println!("----------------------------------------------------------------------------------------------------------------------");
                    // println!("Entry: {:?} Depth: {:?}", entry.path(), entry.depth()  );
                }

                oldDepth  = entry.depth();

                // Dive Up and to the side, depending on Directory 
                for each in ground_vertices { 
                    
                    vertexvec.push(   ( // Rotation * Translation -> Transform TriangleVertex -> into Vec<[f32; 3]>
                                        Affine3A::from_quat(Quat::from_rotation_x(rot.x)*Quat::from_rotation_y(rot.y)*Quat::from_rotation_z(rot.z)) *
                                        Affine3A::from_translation(Vec3{
                                            x: trans.x,
                                            y: trans.y,
                                            z: trans.z })
                                      )
                                      .transform_point3( Vec3::from_array(each) ) 
                                      .into()
                );
                }

                // multiply indizes
                add_indi = 20 * (cnt as u32);
                indexvec.extend(vec![  
                        
                        0+add_indi, 1  +add_indi, 2 +add_indi ,
                        0+add_indi, 2  +add_indi, 3 +add_indi ,
                        0+add_indi, 3  +add_indi, 4 +add_indi ,
        
                        6+add_indi, 5  +add_indi, 3 +add_indi ,
                        6+add_indi, 3  +add_indi, 2 +add_indi , 
                        6+add_indi, 2  +add_indi, 7 +add_indi , 
          
                        2+add_indi, 1  +add_indi, 9 +add_indi ,
                        2+add_indi, 9  +add_indi, 8 +add_indi , 
                        2+add_indi, 8  +add_indi, 7 +add_indi ,
          
                        3+add_indi, 5  +add_indi, 11+add_indi , 
                        3+add_indi, 11 +add_indi, 10+add_indi ,
                        3+add_indi, 10 +add_indi, 4 +add_indi ,
        
                        5+add_indi, 6  +add_indi, 13+add_indi , 
                        5+add_indi, 13 +add_indi, 12+add_indi , 
                        5+add_indi, 12 +add_indi, 11+add_indi , 
        
                        1+add_indi, 0  +add_indi, 16+add_indi , 
                        1+add_indi, 16 +add_indi, 15+add_indi , 
                        1+add_indi, 15 +add_indi, 9 +add_indi , 
 
                        7+add_indi, 8  +add_indi, 14+add_indi , 
                        7+add_indi, 14 +add_indi, 13+add_indi , 
                        7+add_indi, 13 +add_indi, 6 +add_indi , 
        
                        4+add_indi, 10 +add_indi, 17+add_indi , 
                        4+add_indi, 17 +add_indi, 16+add_indi , 
                        4+add_indi, 16 +add_indi, 0 +add_indi , 
        
                        13+add_indi, 14+add_indi, 19+add_indi , 
                        13+add_indi, 19+add_indi, 18+add_indi , 
                        13+add_indi, 18+add_indi, 12+add_indi , 
        
                        16+add_indi, 17+add_indi, 18+add_indi , 
                        16+add_indi, 18+add_indi, 19+add_indi , 
                        16+add_indi, 19+add_indi, 15+add_indi , 
        
                        18+add_indi, 17+add_indi, 10+add_indi , 
                        18+add_indi, 10+add_indi, 11+add_indi , 
                        18+add_indi, 11+add_indi, 12+add_indi , 

                        19+add_indi, 14+add_indi, 8 +add_indi , 
                        19+add_indi, 8 +add_indi, 9 +add_indi , 
                        19+add_indi, 9 +add_indi, 15+add_indi , 

                        ]); 

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
                        trans.x = cntVert * entry.depth() as f32 *0.02;
                        // trans.y = cnt;
                        trans.z = 0.0;

                        // rot.x      = rot.x+1.0;
                        // rot.y      = rot.y+1.0;
                        // rot.z      = rot.z+1.0;

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

        let mut treemesh : Mesh = Mesh::new(PrimitiveTopology::TriangleList);

        treemesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; vertexvec.len()]);
        treemesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 1.]; vertexvec.len()]);
        
        
        println!("vertexvecLen: {}", vertexvec.len());
        println!("indexvecLen: {}", indexvec.len());
        println!("NumberOfEntries: {}", cntAllFiles);

        println!("NumberOfDirectories: {}", cnt);

    
        treemesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vertexvec,
        );

        treemesh.set_indices(Some(mesh::Indices::U32(indexvec)));

        // Handing back the generated mesh
        treemesh.clone()
}
