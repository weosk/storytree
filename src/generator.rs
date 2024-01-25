
use bevy::asset::io::memory::Data;
use bevy::reflect::Enum;
use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::math::*;      // Affine3A
use bevy::prelude::*;   
use bevy::utils::RandomState;
use walkdir::{WalkDir, DirEntry};

use meshtext::{MeshGenerator, MeshText, TextSection, QualitySettings, Face};
use std::collections::LinkedList;
use std::time::Instant;

type PrimitiveTransform = [f32; 16];
// enum MeshType {
//     Text,
//     Space,
// }

pub enum GenerationType {
    Cone, 
    Flat,
    Branch,
    Tree, 
}

// /// A list of lines with a start and end position > https://bevyengine.org/examples/3D%20Rendering/lines/
// #[derive(Debug, Clone)]
// pub struct LineList {
//     pub lines: Vec<(Vec3, Vec3)>,
// }

// impl From<LineList> for Mesh {
//     fn from(line: LineList) -> Self {
//         let vertices: Vec<_> = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();

//         // This tells wgpu that the positions are list of lines
//         // where every pair is a start and end point
//         Mesh::new(PrimitiveTopology::LineList)
//             // Add the vertices positions as an attribute
//             .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
//     }
// }

pub fn walk_path_to_mesh(entry_path: &str, generation_type: GenerationType, textflag: bool) -> (Mesh, Mesh, Mesh) 
{   
    let mut cnt: f32 = 0.;
    let mut text_vertices:  Vec<f32> = vec![];
    let mut space_vertices: Vec<[f32; 3]> = vec![];
    let mut space_indices:  Vec<u32> = vec![];
    let mut line_vertices: Vec<Vec3> = vec![];
    line_vertices.push(Vec3::default());

    let font_data = include_bytes!("/home/nom/code/rust/storytree/assets/fonts/Roboto-Regular.ttf");
    let mut generator = MeshGenerator::new_with_quality(font_data, QualitySettings{quad_interpolation_steps:1,cubic_interpolation_steps:1});
    let common = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".to_string();
    // Precache both flat and three-dimensional glyphs both for indexed and non-indexed meshes.
    generator.precache_glyphs(&common, false, None);
    generator.precache_glyphs(&common, true, None);

    let mut transform;
    for entry in WalkDir::new(entry_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_dir() {
            cnt += 1.;
            match generation_type {
                GenerationType::Cone => 
                    transform = next_cone_transform(cnt),
                GenerationType::Flat => 
                    transform = next_flat_transform(cnt),
                GenerationType::Branch => 
                    transform = next_branch_transform(cnt, &entry),
                GenerationType::Tree => 
                    transform = next_tree_transform(cnt, &entry),
                
            }
            if textflag == true {
                extend_text_vec (&mut text_vertices, &mut generator, transform, &entry);
            }
            extend_space_vec(&mut space_vertices, &mut space_indices, transform, cnt);
            
            // lines currently only show transformation to transformation but should show relation to relation
            line_vertices.push(Mat4::from_cols_array(&transform).transform_point3(Vec3::default()));
            // line_vertices.push(Mat4::from_cols_array(&transform).transform_point3(line_vertices.last().unwrap().clone()));
            // println!("LastLineVert: {}, {}, {}", line_vertices.last().unwrap().x, line_vertices.last().unwrap().y, line_vertices.last().unwrap().z);
        }
    }

    let mut text_mesh : Mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut space_mesh : Mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut line_mesh : Mesh = Mesh::new(PrimitiveTopology::LineStrip);

    let text_uvs =  vec![[0f32, 0f32]; text_vertices.len()];
    let space_uvs = vec![[0f32, 0f32]; space_vertices.len()];

    let text_vertex_positions: Vec<[f32; 3]> = text_vertices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();

    text_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, text_vertex_positions);
    text_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, text_uvs);
    text_mesh.compute_flat_normals();

    space_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, space_vertices.clone());
    space_mesh.set_indices(Some(mesh::Indices::U32(space_indices)));
    space_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, space_uvs);
    space_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, space_vertices); // Normals are just the vertex positions as we go out from 0,0,0

    line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, line_vertices);

    return (text_mesh, space_mesh, line_mesh)
}

fn next_cone_transform(cnt: f32) -> PrimitiveTransform {
    let mut rotation: Vec3 = Default::default();
    let mut translation: Vec3 = Default::default();
    
    translation.y = cnt as f32 *0.2;    
    translation.z =-cnt as f32 *0.2;    
    rotation.y =    cnt as f32;

    generate_primitive_transfrom(rotation, translation)
}

fn next_flat_transform(cnt: f32) -> PrimitiveTransform {
    let mut rotation: Vec3 = Default::default();
    let mut translation: Vec3 = Default::default();
    
    translation.z = -5. -cnt as f32 *0.4;
    rotation.y =    cnt as f32;

    generate_primitive_transfrom(rotation, translation)
}

// Needs arguments to collect information about where we have already been 
// example: /usr/include/libdbusmenu-glib-0.4
// using the characters of the path string, simliar paths end up in the same direction (mostly)
// Problem rigth now: We do a full turn after we adjusted for a direction
fn next_branch_transform(cnt: f32, entry: &DirEntry) -> PrimitiveTransform {

    let mut transform :Mat4 = Default::default();

    // Split path at / into unique folder names
    let words = entry.path().to_str().unwrap().split("/");

    let mut rotation: Vec3 = Default::default();
    let mut translation: Vec3 = Default::default();
    let mut scale: Vec3 = Default::default();

    // Iterate over words and calculate a transform for each one
    for (i, word) in words.enumerate() {

        // Adjusted by number of words
        translation.z -= 3. * i as f32;

        // Prints iteration value and word
        println!("{}: {}",i, word);

        // Enumerates over every char per word and sets the rotation accordingly
        for (j, c) in word.chars().enumerate() {

            // Prints letter by letter with iteration number and ascii value
            print!("{}={}({})",i, c, c as i32);
            translation.y += 4.;
            // translation.z += 4.;
            if i <= 1{                 
                if j == 0 {
                    rotation.y += get_angle(c) * 10.;    
                }
                else {
                    rotation.y += get_angle(c) / 10_f32.powf(j as f32);
                }
            }  
            else 
            {
                translation.z -= 1.;
                rotation.y += get_angle(c) / 10_f32.powf((i+j) as f32);
            }
            
        }

        // Scale experimentation
        scale.x = 1.;// / i as f32;
        scale.y = 1.;// / i as f32;
        scale.z = 1.;// / i as f32;
        // let base:f32 = 0.9;
        // if i >= 7{                 
        //     let scalf = base.powf(i as f32);//0.9 * i as f32;
        //     scale.x = scalf;// / i as f32;
        //     scale.y = scalf;// / i as f32;
        //     scale.z = scalf;// / i as f32;
        // }

        // Stack unique word transforms together for full path transform
        transform *= Mat4::from_rotation_y(rotation.y) * Mat4::from_translation(translation) * Mat4::from_scale(scale);
    }
    // creates PrimitiveTransform aka [f32,16] and returns it
    transform.to_cols_array() 
}

// Take lessons from branching and create nested nests
fn next_tree_transform(cnt: f32, entry: &DirEntry) -> PrimitiveTransform{
    let mut transform :Mat4 = Default::default();
    transform.to_cols_array()
}

fn get_angle(c: char) -> f32
{
    let mut angle: f32 = 0.;
    match c {
        'a' | 'A' => angle =  1.,
        'b' | 'B' => angle =  2.,
        'c' | 'C' => angle =  3.,
        'd' | 'D' => angle =  4.,
        'e' | 'E' => angle =  5.,
        'f' | 'F' => angle =  6.,
        'g' | 'G' => angle =  7.,
        'h' | 'H' => angle =  8.,
        'i' | 'I' => angle =  9.,
        'j' | 'J' => angle = 10.,
        'k' | 'K' => angle = 11.,
        'l' | 'L' => angle = 12.,
        'm' | 'M' => angle = 13.,
        'n' | 'N' => angle = 14.,
        'o' | 'O' => angle = 15.,
        'p' | 'P' => angle = 16.,
        'q' | 'Q' => angle = 17.,
        'r' | 'R' => angle = 18.,
        's' | 'S' => angle = 19.,
        't' | 'T' => angle = 20.,
        'u' | 'U' => angle = 21.,
        'v' | 'V' => angle = 22.,
        'x' | 'X' => angle = 23.,
        'y' | 'Y' => angle = 24.,
        'z' | 'Z' => angle = 25.,
        '0'       => angle = 26.,
        '1'       => angle = 27.,
        '2'       => angle = 28.,
        '3'       => angle = 29.,
        '4'       => angle = 30.,
        '5'       => angle = 31.,
        '6'       => angle = 32.,
        '7'       => angle = 33.,
        '8'       => angle = 34.,
        '9'       => angle = 35.,
        _ => angle = 0.,
    } 

    angle
}

// Generates the 3D text and offsets it to be readable 
fn extend_text_vec(vertices: &mut Vec<f32>, generator: &mut MeshGenerator<Face>, mut transform: [f32; 16], entry: &DirEntry) {
    // Adjust position, relative to dodeca
    // transform[12] -= 1.; // x
    transform[13] += 1.7; // y
    // transform[14] += 2.; // z

    let text_mesh: MeshText = generator
        .generate_section(entry.path().to_str().unwrap(), true, Some(&transform))
        .unwrap();

    vertices.extend(text_mesh.vertices);
}

// Creates the dodecas
fn extend_space_vec(space_vertices: &mut Vec<[f32; 3]>, space_indices: &mut Vec<u32>, transform: PrimitiveTransform, cnt: f32){
    let PHI: f32 = 1.618033989; 
    let ground_vertices: [[f32; 3]; 20] =   
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

    // multiply indizes
    let add_indi = 20 * (cnt as u32);
    space_indices.extend(vec![  
                
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

        // Convert the transformation matrix to Mat4
        let transform_matrix = Mat4::from_cols_array(&transform);

        // Create a Bevy Transform component
        let transform = Transform::from_matrix(transform_matrix);
    
        for each in ground_vertices {
            // Convert each input vector to Vec3
            let position_vector = Vec3::new(each[0], each[1], each[2]);
    
            // Perform the vector transformation using Bevy's Transform component
            let transformed_vector = transform.transform_point(position_vector);
    
            // Push the transformed vector into vertexvec
            space_vertices.push(transformed_vector.into());
        }
}

fn count_directories(path: &str) -> i32{
    let mut cnt = -1;
    for entry in WalkDir::new(path).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_dir() {
            cnt += 1; 
        }} cnt 
}

// replace rotation with Vec3 when working, only uses Y right now
fn generate_primitive_transfrom(roation: Vec3, translation: Vec3) -> PrimitiveTransform {
    (Mat4::from_rotation_y(-roation.y) * Mat4::from_translation(translation))
    .to_cols_array()
}






















// Old Spice

// pub fn generate_space_mesh(
//     //mut meshTreedata: &mut ResMut<Treedata>,
// ) -> Mesh {
//         let PHI: f32 = 1.618033989; 

//         // Vertices
//         // Plain Data dodecaeder
//         let mut ground_vertices: [[f32; 3]; 20] =   
//             [   [  0.,      -1./PHI,  -PHI ], // 0
//                 [  1.,      -1.,      -1.  ], // 1
//                 [  1./PHI,  -PHI,     0.   ], // 2
//                 [  -1./PHI, -PHI,     0.   ], // 3
//                 [  -1.,     -1.,      -1.  ], // 4

//                 [  -1.,      -1.,      1.  ], // 5
//                 [  0.,      -1./PHI,  PHI  ], // 6
//                 [  1.,      -1.,      1.   ], // 7

//                 [  PHI,      0.,    1./PHI ], // 8
//                 [  PHI,      0.,   -1./PHI ], // 9
//                 [  -PHI,     0.,   -1./PHI ], // 10
//                 [  -PHI,     0.,    1./PHI ], // 11

//                 // Gespiegelt an der XZ Ebene, -> Werte von 0 - 7 mit positiver y achse

//                 [  -1.,      1.,      1.  ], // 12
//                 [  0.,       1./PHI,  PHI  ], // 13
//                 [  1.,       1.,      1.   ], // 14
//                 [  1.,       1.,      -1.  ], // 15
//                 [  0.,       1./PHI,  -PHI ], // 16
//                 [  -1.,      1.,      -1.  ], // 17
//                 [  -1./PHI,  PHI,     0.   ], // 18
//                 [  1./PHI,   PHI,     0.   ]  // 19 
//                 ];

//         let ground_indices = [         
//                 0, 1, 2,
//                 0, 2, 3,
//                 0, 3, 4,

//                 6, 5, 3,
//                 6, 3, 2, 
//                 6, 2, 7,

//                 2, 1, 9,
//                 2, 9, 8, 
//                 2, 8, 7,

//                 3, 5, 11, 
//                 3, 11, 10,
//                 3, 10, 4,

//                 5, 6, 13, 
//                 5, 13, 12, 
//                 5, 12, 11, 

//                 1, 0, 16, 
//                 1, 16, 15, 
//                 1, 15, 9, 

//                 // Halber Dodecaeder Formuliert in symmetrie, für möglichkeit zur animierten öffnung

//                 7, 8, 14, 
//                 7, 14, 13, 
//                 7, 13, 6, 

//                 4, 10, 17, 
//                 4, 17, 16, 
//                 4, 16, 0, 

//                 13, 14, 19, 
//                 13, 19, 18, 
//                 13, 18, 12, 

//                 16, 17, 18, 
//                 16, 18, 19, 
//                 16, 19, 15, 

//                 18, 17, 10, 
//                 18, 10, 11, 
//                 18, 11, 12, 

//                 19, 14, 8, 
//                 19, 8, 9, 
//                 19, 9, 15, 
//             ];

//             // Transformations Matrix
//             let tmat:Affine3A = Affine3A::from_translation(Vec3{x:0.0,y:1.0,z:0.0}.into());

//             let mut vertexvec: Vec<[f32; 3]> = vec![];
//             let mut indexvec: Vec<u32> = vec![];

//             let mut uvvec: Vec<[f32;2]> = vec![];


//             let mut cnt = 0.0; 
//             let mut cntOld = 0.0;
//             let mut cntVert = 0.0;
//             // let mut rot = 0.0;
        
//             let mut countVertices = 0;
        
//             let mut add_indi: u32 = 0;


//             let mut rot : Vec3 = Vec3{x:0.0,
//                                       y:1.0,
//                                       z:0.0};

//             let mut trans : Vec3 = Vec3{x:0.0,
//                                         y:1.0,
//                                         z:0.0};

//             let mut oldDepth = 0;

//             let mut cntAllFiles = 0; // was once 4 330 659
//                                      // dirs then: 430 573
//                                      // Soo we print dir names to texture and bind as uv? 
//                                      // and number of files holding gets represeneted how? 
                                     
//             for entry in WalkDir::new("/").into_iter().filter_map(|e| e.ok()) {        
//             // for entry in WalkDir::new("./TestTree").into_iter().filter_map(|e| e.ok()) {
//                 // println!("Entry: {:?} Depth: {:?}", entry.path(), entry.depth()  );
                

//                 if entry.file_type().is_dir() 
//                 {
                    
//                 countVertices += 20;


//                 // println!("Entry: {:?} Depth: {:?}", entry.path(), entry.depth()  );


//                 // Depth \
//                 if oldDepth < entry.depth() 
//                 {
//                     rot.y      = rot.y+1.0;
//                     trans.y = trans.y + 1.;

//                     // println!("----------------------------------------------------------------------------------------------------------------------");
//                     // println!("Entry: {:?} Depth: {:?}", entry.path(), entry.depth()  );
//                 }

//                 oldDepth  = entry.depth();

//                 // Dive Up and to the side, depending on Directory 
//                 for each in ground_vertices { 
                    
//                     vertexvec.push(   ( // Rotation * Translation -> Transform TriangleVertex -> into Vec<[f32; 3]>
//                                         Affine3A::from_quat(Quat::from_rotation_x(rot.x)*Quat::from_rotation_y(rot.y)*Quat::from_rotation_z(rot.z)) *
//                                         Affine3A::from_translation(Vec3{
//                                             x: trans.x,
//                                             y: trans.y,
//                                             z: trans.z })
//                                       )
//                                       .transform_point3( Vec3::from_array(each) ) // Each Ground Vertex gets pushed where it should go
//                                       .into()
//                 );
//                 }

//                 // multiply indizes
//                 add_indi = 20 * (cnt as u32);
//                 indexvec.extend(vec![  
                        
//                         0+add_indi, 1  +add_indi, 2 +add_indi ,
//                         0+add_indi, 2  +add_indi, 3 +add_indi ,
//                         0+add_indi, 3  +add_indi, 4 +add_indi ,
        
//                         6+add_indi, 5  +add_indi, 3 +add_indi ,
//                         6+add_indi, 3  +add_indi, 2 +add_indi , 
//                         6+add_indi, 2  +add_indi, 7 +add_indi , 
          
//                         2+add_indi, 1  +add_indi, 9 +add_indi ,
//                         2+add_indi, 9  +add_indi, 8 +add_indi , 
//                         2+add_indi, 8  +add_indi, 7 +add_indi ,
          
//                         3+add_indi, 5  +add_indi, 11+add_indi , 
//                         3+add_indi, 11 +add_indi, 10+add_indi ,
//                         3+add_indi, 10 +add_indi, 4 +add_indi ,
        
//                         5+add_indi, 6  +add_indi, 13+add_indi , 
//                         5+add_indi, 13 +add_indi, 12+add_indi , 
//                         5+add_indi, 12 +add_indi, 11+add_indi , 
        
//                         1+add_indi, 0  +add_indi, 16+add_indi , 
//                         1+add_indi, 16 +add_indi, 15+add_indi , 
//                         1+add_indi, 15 +add_indi, 9 +add_indi , 
 
//                         7+add_indi, 8  +add_indi, 14+add_indi , 
//                         7+add_indi, 14 +add_indi, 13+add_indi , 
//                         7+add_indi, 13 +add_indi, 6 +add_indi , 
        
//                         4+add_indi, 10 +add_indi, 17+add_indi , 
//                         4+add_indi, 17 +add_indi, 16+add_indi , 
//                         4+add_indi, 16 +add_indi, 0 +add_indi , 
        
//                         13+add_indi, 14+add_indi, 19+add_indi , 
//                         13+add_indi, 19+add_indi, 18+add_indi , 
//                         13+add_indi, 18+add_indi, 12+add_indi , 
        
//                         16+add_indi, 17+add_indi, 18+add_indi , 
//                         16+add_indi, 18+add_indi, 19+add_indi , 
//                         16+add_indi, 19+add_indi, 15+add_indi , 
        
//                         18+add_indi, 17+add_indi, 10+add_indi , 
//                         18+add_indi, 10+add_indi, 11+add_indi , 
//                         18+add_indi, 11+add_indi, 12+add_indi , 

//                         19+add_indi, 14+add_indi, 8 +add_indi , 
//                         19+add_indi, 8 +add_indi, 9 +add_indi , 
//                         19+add_indi, 9 +add_indi, 15+add_indi , 

//                         ]); 

//                     uvvec.extend(vec![  [-0.1,0.5], //0
//                                         [0.1,1.],  //1

//                                         [0.5,1.], //2
//                                         [0.5,0.], //3
                                        
//                                         [0.1,0.],  //4

//                                         [0.9,0.],  //5
//                                         [1.1,0.5], //6
//                                         [0.9,1.],   //7


//                                         [0.,1.], //8
//                                         [0.,1.], //9
//                                         [1.,0.], //10
//                                         [1.,0.], //11


//                                         [0.1,0.],  //12
//                                         [-0.1,0.5],//13
//                                         [0.1,1.],  //14

//                                         [0.9,1.],   //15
//                                         [1.1,0.5], //16

//                                         [0.9,0.],  //17
//                                         [0.5,0.], //18
//                                         [0.5,1.], //19

//                                     ]);

//                 // println!("Dir: {:?}", entry.path());
//                 // print!("Dir: {:?} Depth: {:?}", entry.path(), entry.depth() );
//                 // println!();

//                 cnt += 1.0;
//                 }
//                 else if entry.file_type().is_file() 
//                 {
//                     cntVert += 1.0; 
//                     // println!("Filename: {:?} \n {:?} \n", entry.file_name(), entry.metadata());
        
//                     if cnt != cntOld 
//                     {
//                         trans.x = cntVert * entry.depth() as f32 *0.02;
//                         // trans.y = cnt;
//                         trans.z = 0.0;

//                         // rot.x      = rot.x+1.0;
//                         // rot.y      = rot.y+1.0;
//                         // rot.z      = rot.z+1.0;

//                         cntVert *= -1.0;
//                         cntOld   = cnt; 
//                     }
//                 }
//                 else if entry.file_type().is_symlink()
//                 {
//                     // println!("Symlink: {:?}", entry.file_name());
//                 }
//                 else 
//                 {
//                     //println!("Not Dir nor File: {:?}", entry.file_name());
//                 }
//             }
    
        

//         println!("Verti: {}", countVertices);

//         let mut treemesh : Mesh = Mesh::new(PrimitiveTopology::TriangleList);

//         treemesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; vertexvec.len()]);
//         treemesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvvec);

//         // treemesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![
//         //     [-0.1,0.5], //0
//         //     [0.1,1.],  //1

//         //     [0.5,1.], //2
//         //     [0.5,0.], //3
            
//         //     [0.1,0.],  //4

//         //     [0.9,0.],  //5
//         //     [1.1,0.5], //6
//         //     [0.9,1.],   //7


//         //     [0.,1.], //8
//         //     [0.,1.], //9
//         //     [1.,0.], //10
//         //     [1.,0.], //11


//         //     [0.1,0.],  //12
//         //     [-0.1,0.5],//13
//         //     [0.1,1.],  //14

//         //     [0.9,1.],   //15
//         //     [1.1,0.5], //16

//         //     [0.9,0.],  //17
//         //     [0.5,0.], //18
//         //     [0.5,1.], //19


//         //     ]);


//         // treemesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![
//         //     [-0.1,0.5], //0
//         //     [0.1,1.],  //1

//         //     [0.5,1.], //2
//         //     [0.5,0.], //3
            
//         //     [0.1,0.],  //4

//         //     [0.9,0.],  //5
//         //     [1.1,0.5], //6
//         //     [0.9,1.],   //7


//         //     [0.,1.], //8
//         //     [0.,1.], //9
//         //     [1.,0.], //10
//         //     [1.,0.], //11


//         //     [0.1,0.],  //12
//         //     [-0.1,0.5],//13
//         //     [0.1,1.],  //14

//         //     [0.9,1.],   //15
//         //     [1.1,0.5], //16

//         //     [0.9,0.],  //17
//         //     [0.5,0.], //18
//         //     [0.5,1.], //19


//         //     ]);
//         // [1.,0.],
//         // [1.,1.],
//         // [0.,1.],

//         // [0., 0.],
//         // [-1.5, 0.],
//         // [1.,0.],
//         // [1.,1.],
//         // [0.,1.],

//         // [0., 0.],
//         // [-1.5, 0.],
//         // [1.,0.],
//         // [1.,1.],
//         // [0.,1.],

//         // [0., 0.],
//         // [-1.5, 0.],
//         // [1.,0.],
//         // [1.,1.],
//         // [0.,1.],

//         // [0., 0.],
//         // [-1.5, 0.],
//         // [1.,0.],
//         // [1.,1.],
//         // [0.,1.]

//         // ]);

//         println!("vertexvecLen: {}", vertexvec.len());
//         println!("indexvecLen: {}", indexvec.len());
//         println!("NumberOfEntries: {}", cntAllFiles);

//         println!("NumberOfDirectories: {}", cnt);

    
//         treemesh.insert_attribute(
//             Mesh::ATTRIBUTE_POSITION,
//             vertexvec,
//         );

//         treemesh.set_indices(Some(mesh::Indices::U32(indexvec)));

//         // Handing back the generated mesh
//         treemesh.clone()
// }



// // pub fn generate_text_mesh(
// //     path: &str,
// //     //mut meshTreedata: &mut ResMut<Treedata>,
// // ) -> Mesh {

// //     let before = Instant::now();

// //     let font_data = include_bytes!("/home/nom/code/rust/storytree/assets/fonts/Roboto-Regular.ttf");
// //     let mut generator = MeshGenerator::new_with_quality(font_data, QualitySettings{quad_interpolation_steps:1,cubic_interpolation_steps:1});

// //     let common = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".to_string();
// //     // Precache both flat and three-dimensional glyphs both for indexed and non-indexed meshes.
// //     generator.precache_glyphs(&common, false, None);
// //     generator.precache_glyphs(&common, true, None);

// //     // let mut vertexvec: Vec<[f32; 3]> = vec![];
// //     let mut collection = vec![];
// //     let mut cnt = 0;
// //     let mut depthOld = 0;
// //     let mut xOff = 0.;
// //     let mut rot = 0.;
// //     for entry in WalkDir::new(path).max_depth(0).into_iter().filter_map(|e| e.ok()) {        
// //             if entry.file_type().is_dir() 
// //             {   
// //                 // if depthOld != entry.depth()
// //                 // {
// //                 //     if depthOld < entry.depth() {
// //                 //         xOff += 0.1;
// //                 //     }
// //                 //     if depthOld > entry.depth() {
// //                 //         rot += 0.1;
// //                 //     }
// //                 //     depthOld = entry.depth();
// //                 // }
// //                 // cnt += 1;
// //                 cnt = -1;
// //                 for local_entry in WalkDir::new(entry.path()).max_depth(1).into_iter().filter_map(|e| e.ok()) {    
// //                     if local_entry.file_type().is_dir() 
// //                     {   
// //                         cnt+=1;
// //                         println!("Number of directories in {:?}: {}",local_entry.path(), cnt);
// //                     }
// //                 }

// //                 println!("Number of directories in {}: {}",entry.path().display(), cnt);

// //                 let transform = (Mat4::from_rotation_y(-rot) * Mat4::from_translation(Vec3::new(cnt as f32 *0.2, entry.depth() as f32 * 5., -xOff as f32 * 0.5))).to_cols_array();

// //                 let text_mesh: MeshText = generator
// //                     .generate_section(entry.file_name().to_str().unwrap(), true, Some(&transform))
// //                     .unwrap();

// //                 collection.extend(text_mesh.vertices);
// //             }
// //     }

// //     let vertices = collection;
// //     let positions: Vec<[f32; 3]> = vertices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();


// //     println!("Poslength: {}", positions.len());


// //     let uvs = vec![[0f32, 0f32]; positions.len()];

// //     let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
// //     mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
// //     mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
// //     mesh.compute_flat_normals();

// //     println!("Text generation time: {:?}", before.elapsed());

// //     mesh

// // }

// pub fn generate_text_mesh(
//     path: &str,
//     //mut meshTreedata: &mut ResMut<Treedata>,
// ) -> Mesh {

//     let before = Instant::now();

//     let font_data = include_bytes!("/home/nom/code/rust/storytree/assets/fonts/Roboto-Regular.ttf");
//     let mut generator = MeshGenerator::new_with_quality(font_data, QualitySettings{quad_interpolation_steps:1,cubic_interpolation_steps:1});

//     let common = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".to_string();
//     // Precache both flat and three-dimensional glyphs both for indexed and non-indexed meshes.
//     generator.precache_glyphs(&common, false, None);
//     generator.precache_glyphs(&common, true, None);

//     // let mut vertexvec: Vec<[f32; 3]> = vec![];
//     let mut collection = vec![];
//     let mut cnt = 0;
//     let mut depthOld = 0;
//     let mut xOff = 0.;
//     let mut zOff = 10.;
//     let mut rot = 0.;
//     // for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {        
//     for entry in WalkDir::new("/home/nom").into_iter().filter_map(|e| e.ok()) {        
//             if entry.file_type().is_dir() 
//             {   
//                 if depthOld != entry.depth()
//                 {
//                     if depthOld < entry.depth() {
//                     }
//                     if depthOld > entry.depth() {
//                         rot += 0.1;
//                         xOff += 1.;
//                         zOff += 1.;
//                         cnt = zOff as i32;
//                     }
//                     println!("Olddepth: {:?} Newdepth: {:?} xOff: {:?} cnt: {:?}", depthOld, entry.depth(), xOff, cnt);
//                     depthOld = entry.depth();
//                 }
//                 cnt += 1;
//                 let transform = (Mat4::from_rotation_y(-rot) * Mat4::from_translation(Vec3::new(
//                     xOff * 1.0 , 
//                     entry.depth() as f32 * 2. + cnt  as f32* 0.2, 
//                     -cnt as f32 * 0.4
//                 ))).to_cols_array();

//                 let text_mesh: MeshText = generator
//                     .generate_section(entry.file_name().to_str().unwrap(), true, Some(&transform))
//                     .unwrap();

//                 collection.extend(text_mesh.vertices);
//             }
//     }

//     let vertices = collection;
//     let positions: Vec<[f32; 3]> = vertices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();


//     println!("Poslength: {}", positions.len());
    
//     let uvs = vec![[0f32, 0f32]; positions.len()];

//     let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
//     mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
//     mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
//     mesh.compute_flat_normals();

//     println!("Text generation time: {:?}", before.elapsed());

//     // println!("#: {}", count_dirs("/home/nom") );

//     mesh

// }

