
use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::math::*;   
use bevy::prelude::*;   
use bevy::render::render_asset::RenderAssetUsages;
use walkdir::{WalkDir, DirEntry};
use meshtext::{MeshGenerator, MeshText, TextSection, QualitySettings, Face};

type PrimitiveTransform = [f32; 16];

pub enum GenerationType {
    Cone, 
    Flat,
    Branch,
    Tree, 
}

pub fn walk_path_to_mesh(entry_path: &str, generation_type: GenerationType, depth: usize, textflag: bool, dodecaflag: bool) -> (Mesh, Mesh, Mesh) 
{   
    let mut cnt: f32 = 0.;
    let mut text_vertices:  Vec<f32> = vec![];
    let mut space_vertices: Vec<[f32; 3]> = vec![];
    let mut space_indices:  Vec<u32> = vec![];
    let mut line_vertices: Vec<Vec3> = vec![];
    let mut line_list_vertices: Vec<Vec3> = vec![];
    let mut line_strip_vertices: Vec<Vec3> = vec![];

    let font_data = include_bytes!("../assets/fonts/Roboto-Regular.ttf");
    let mut generator = MeshGenerator::new_with_quality(font_data, 
        QualitySettings{quad_interpolation_steps:1,cubic_interpolation_steps:1});
    let common = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789/-+?".to_string();
    // Precache both flat and three-dimensional glyphs both for indexed and non-indexed meshes.
    _ = generator.precache_glyphs(&common, false, None);
    _ = generator.precache_glyphs(&common, true, None);
    
    let mut transform: Mat4;
    let mut parent_transform: Mat4; 

    for entry in WalkDir::new(entry_path).max_depth(depth).sort_by
        (|a,b| a.file_name().cmp(b.file_name())).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_dir() {
            cnt += 1.;
            match generation_type {
                GenerationType::Cone => 
                    (transform, parent_transform) = (next_cone_transform(cnt), Mat4::default()),
                GenerationType::Flat => 
                    (transform, parent_transform) = (next_flat_transform(cnt), Mat4::default()),
                GenerationType::Branch => 
                    (transform, parent_transform) = (next_branch_transform(&entry.path().to_str().unwrap()), calc_parent_transform(&entry.path().to_str().unwrap())),
                GenerationType::Tree => 
                    (transform, parent_transform) = (next_tree_transform(cnt, &entry), Mat4::default())
            }
            if textflag == true {
                extend_text_vec (&mut text_vertices, &mut generator, &transform, &entry);
            }
            if dodecaflag == true {
                extend_space_vec(&mut space_vertices, &mut space_indices, &transform, cnt);
            }
            extend_line_list_vec(&mut line_list_vertices, &transform, &parent_transform);

            // Calculates sets of lines
            // LineList: Branches from (close to start) to far
            // LineStrip: Order of transforms 
            extend_line_list_vec(&mut line_list_vertices, &transform, &parent_transform);
        }
    }

    let mut text_mesh : Mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut space_mesh : Mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    let mut line_mesh : Mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());

    let text_uvs =  vec![[0f32, 0f32]; text_vertices.len() ];
    let space_uvs = vec![[0f32, 0f32]; space_vertices.len()];

    let text_vertex_positions: Vec<[f32; 3]> = text_vertices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();

    text_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, text_vertex_positions);
    text_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, text_uvs);
    text_mesh.compute_flat_normals();

    space_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, space_vertices.clone());
    space_mesh.insert_indices(mesh::Indices::U32(space_indices));
    space_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, space_uvs);
    space_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, space_vertices); // Normals are just the vertex positions as we go out from 0,0,0

    // let uvs = vec![[0.5, 0.5]; line_list_vertices.len()];
    line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, line_list_vertices);
    // space_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    return (text_mesh, space_mesh, line_mesh)
}

fn next_cone_transform(cnt: f32) -> Mat4 {
    let mut rotation: Vec3 = Default::default();
    let mut translation: Vec3 = Default::default();
    
    translation.y = cnt as f32 *0.2;    
    translation.z =-cnt as f32 *0.2;    
    rotation.y =    cnt as f32;

    // This would be the "correct" way, but we would loose rotation relocalisation
    // Mat4::from_rotation_translation(calc_quat_from_rot_vec(rotation), translation)
    Mat4::from_rotation_y(rotation.y) * Mat4::from_translation(translation)
}

fn next_flat_transform(cnt: f32) -> Mat4 {
    let mut rotation: Vec3 = Default::default();
    let mut translation: Vec3 = Default::default();
    
    translation.z = -5. -cnt as f32 *0.4;
    rotation.y =    cnt as f32;

    // This would be the "correct" way, but we would loose rotation relocalisation
    // Mat4::from_rotation_translation(calc_quat_from_rot_vec(rotation), translation)
    Mat4::from_rotation_y(rotation.y) * Mat4::from_translation(translation)
}

fn calc_quat_from_rot_vec(rotation: Vec3) -> Quat {
    Quat::from_rotation_x(rotation.x) * Quat::from_rotation_y(rotation.y) * Quat::from_rotation_z(rotation.z)
}

fn calc_parent_transform(path: &str) -> Mat4 {
    let parent_string = match path.rsplit_once("/") {
        Some(cut_path) => cut_path.0.to_string() ,
        None    => "/".to_string(),
    };

    // println!("{:?}", parent_string);

    next_branch_transform(&parent_string)
}

// Needs arguments to collect information about where we have already been 
// example: /usr/include/libdbusmenu-glib-0.4
// using the characters of the path string, simliar paths end up in the same direction (mostly)
fn next_branch_transform(path: &str) -> Mat4 {

    let mut transform :Mat4 = Default::default();

    // Split path at / into unique folder names -> creating an iterator over every

    let mut rotation: Vec3 = Default::default();
    let mut translation: Vec3 = Default::default();
    let mut scale: Vec3 = Default::default();

    let dirs = path.split("/");

    if true {

    // Iterate over dirs and calculate a transform for each one / word iterater == depth, angle given through string
    for (i, dir) in dirs.enumerate() {
        
        if !dir.is_empty(){
            translation.z -= 4.;
            translation.y += 1.;

            // Enumerates over every char per word and sets the rotation accordingly
            for (j, c) in dir.chars().enumerate() {
                translation.z += 0.4;
                rotation.y = get_angle(rotation.y, c, i, j) *2.;
                translation.y += 0.2; // Every letter leads to slightly higher position
            }
        }      

        // // Scale experimentation
        scale = Vec3::splat(0.5);
        let base:f32 = 0.9;
        if i >= 2{                 
            let scalf = base.powf(i as f32);
            scale = Vec3::splat(scalf);
        }

        // Stack unique word transforms together for full path transform // Normal Way would be L = T * R * S  -> Order is S then R then T, but we use angletravel
        transform *= Mat4::from_rotation_y(rotation.y) * Mat4::from_rotation_x(rotation.x)  *  Mat4::from_translation(translation) * Mat4::from_scale(scale);
    }
    }

    // 2. Treetrunk // for / this puts out a bit of split, for experimentation
     else if false{ // more space for experimentation
        // Todo: Get them branches to spread out wide first, then decrease their spread so they stay over their local parent folders
        // > z high, then exponentially lower should do the trick? Rotation might need fixing. 
        let cnt_dirs = dirs.clone().count();

        for (i, dir) in dirs.enumerate() {
            if !dir.is_empty(){
                for (j, c) in dir.chars().enumerate() {
                    if i <= 1 { // First step/word only goes up
                        translation.y += c as i32 as f32 * 1.5 - 150.;
                        rotation.y = get_angle(rotation.y, c, i, j);           
                    }    
                    else if i <= 5 {
                        // translation.y += c as i32 as f32 / 20.;
                        translation.y = 0.;
                        translation.z -= (j*2) as f32 * 0.2;//c as i32 as f32 / 20.;//1000. / (2*i+1) as f32;//c as i32 as f32/10.;
                        rotation.y = get_angle(rotation.y, c, i, j);           
                        rotation.x += 0.002;
                    }
                    else 
                    {
                        translation.y = 0.;
                        translation.z -= (j*2) as f32 * 0.4;//c as i32 as f32 / 20.;//1000. / (2*i+1) as f32;//c as i32 as f32/10.;
                        rotation.y = get_angle(rotation.y, c, i, j);           
                        rotation.x -= 0.002;
                    }
                }
            }
            // Scale experimentation
            scale.x = 0.8;// / i as f32;
            scale.y = 0.8;// / i as f32;
            scale.z = 0.8;// / i as f32;
            let base:f32 = 0.95;
            // if i >= 4{                 
            let scalf = base.powf(i as f32);//0.9 * i as f32;
            scale.x *= scalf;// / i as f32;
            scale.y *= scalf;// / i as f32;
            scale.z *= scalf;// / i as f32;
            // }
 
            transform *= Mat4::from_rotation_x(rotation.x) * Mat4::from_rotation_y(rotation.y) * Mat4::from_translation(translation)  * Mat4::from_scale(scale);
        }
    }
    //3. Log naturalis, Bush
    else if false {
        // let cnt_dirs = dirs.clone().count();

        for (i, dir) in dirs.enumerate() {
            if !dir.is_empty(){
                for (j, c) in dir.chars().enumerate() {
                    translation.y += c as i32 as f32 / 30.;
                    translation.z -= (i+j) as f32 * 0.5;//c as i32 as f32 / 20.;//1000. / (2*i+1) as f32;//c as i32 as f32/10.;
                    rotation.y = get_angle(rotation.y, c, i, j);           
                }
            }

            // Scale experimentation
            scale.x = 1.;// / i as f32;
            scale.y = 1.;// / i as f32;
            scale.z = 1.;// / i as f32;
            let base:f32 = 1.1;
            // let base:f32 = translation.z.ln();
            // println!("Ln(y): {:?}, y: {:?}", translation.y.ln(), translation.y);

            // if i >= 4{                 
            let scalf = base.powf(i as f32);//* 1.1 * i as f32;
            scale.x *= scalf;// / i as f32;
            scale.y *= scalf;// / i as f32;
            scale.z *= scalf;// / i as f32;
            
            // if translation.y != 0. {
            //     let scalf = translation.y.ln();//* 1.1 * i as f32;
            //     scale.x = scalf;// / i as f32;
            //     scale.y = scalf;// / i as f32;
            //     scale.z = scalf;// / i as f32;  
            // }

            transform *= /*Mat4::from_rotation_x(rotation.x) */ Mat4::from_rotation_y(rotation.y) * Mat4::from_translation(translation)  * Mat4::from_scale(scale);
            }
    }

    transform
}

// Take lessons from branching and create nested nests
fn next_tree_transform(_cnt: f32, _entry: &DirEntry) -> Mat4{
    let transform :Mat4 = Default::default();
    transform
}
    
// Include Range, governed by count directories, split to 360 / numDirs - Adjusted by alphanumeric value in new range
fn get_angle(mut current_angle: f32, c: char, word_num: usize, _char_num: usize) -> f32
{
    // Map char to char range as integer, map that integer to angle_range
    // 1. 0..2PI
    // 2. 2PI/NumDirectories == > New allowed angle? +- angle_range? 

    // let mut angle: f32 = 0.;
    let min_angle: f32 = 0.1745329252;// (2 * PI) / 36

    let map_pos: i32;
    match c {
        'a' | 'A' => map_pos = -17,
        'b' | 'B' => map_pos = -16,
        'c' | 'C' => map_pos = -15,
        'd' | 'D' => map_pos = -14,
        'e' | 'E' => map_pos = -13,
        'f' | 'F' => map_pos = -12,
        'g' | 'G' => map_pos = -11,
        'h' | 'H' => map_pos = -10,
        'i' | 'I' => map_pos = -9,
        'j' | 'J' => map_pos = -8,
        'k' | 'K' => map_pos = -7,
        'l' | 'L' => map_pos = -6,
        'm' | 'M' => map_pos = -5,
        'n' | 'N' => map_pos = -4,
        'o' | 'O' => map_pos = -3,
        'p' | 'P' => map_pos = -2,
        'q' | 'Q' => map_pos = -1,
        'r' | 'R' => map_pos =  0,
        's' | 'S' => map_pos =  1,
        't' | 'T' => map_pos =  2,
        'u' | 'U' => map_pos =  3,
        'v' | 'V' => map_pos =  4,
        'x' | 'X' => map_pos =  5,
        'y' | 'Y' => map_pos =  6,
        'z' | 'Z' => map_pos =  7,
        '0'       => map_pos =  8,
        '1'       => map_pos =  9,
        '2'       => map_pos =  10,
        '3'       => map_pos =  11,
        '4'       => map_pos =  12,
        '5'       => map_pos =  13,
        '6'       => map_pos =  14,
        '7'       => map_pos =  15,
        '8'       => map_pos =  16,
        '9'       => map_pos =  17,
        _         => map_pos =  40 - c as i32,
    } 

    if word_num <= 1{

    // if word_num <= 2 { //first word, initial angle
    //    angle = map_pos as f32 * min_angle;
    // }
    // else { 
    //     current_angle = current_angle - min_angle / 2.;
    //     current_angle += min_angle / map_pos as f32;
    //     angle = current_angle;
    // }

        current_angle += map_pos as f32 * min_angle; // (10. * (char_num+1) as f32);
        // angle += angle.powf((word_num*char_num )as f32);
        current_angle
    }
    else {
        current_angle = (map_pos as f32 * min_angle) * 0.5f32.powf(word_num as f32); // (10. * (word_num + char_num) as f32);
        current_angle
    }      

}

// Generates the 3D text and offsets it to be readable 
fn extend_text_vec(vertices: &mut Vec<f32>, generator: &mut MeshGenerator<Face>, transform: &Mat4, entry: &DirEntry) {

    // Adjust position, relative to dodeca, adjust scale
    let transform = transform.clone() * Mat4::from_translation(Vec3 { x: 0.1, y: 0.1, z: 0. }) * Mat4::from_scale(Vec3::splat(0.5));

    let text_mesh: MeshText = generator
        .generate_section(entry.path().to_str().unwrap(), true, Some(&transform.to_cols_array()))
        .unwrap();

    vertices.extend(text_mesh.vertices);
}

// Creates the dodecaeder
pub fn extend_space_vec(space_vertices: &mut Vec<[f32; 3]>, space_indices: &mut Vec<u32>, transform: &Mat4, cnt: f32){
    const PHI: f32 = 1.618033989; 
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

        [  -1.,      1.,      1.   ], // 12
        [  0.,       1./PHI,  PHI  ], // 13
        [  1.,       1.,      1.   ], // 14
        [  1.,       1.,      -1.  ], // 15
        [  0.,       1./PHI,  -PHI ], // 16
        [  -1.,      1.,      -1.  ], // 17
        [  -1./PHI,  PHI,     0.   ], // 18
        [  1./PHI,   PHI,     0.   ]  // 19 
        ];

    // Multiply indizes
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
        // let transform_matrix = Mat4::from_cols_array(&transform);

        //info!("Scale: {}",*transform);

        // Create a Bevy Transform component
        let transform = Transform::from_matrix(*transform);//.clone());
        for each in ground_vertices {
            // Convert each input vector to Vec3
            let position_vector = Vec3::new(each[0], each[1], each[2])*Vec3::splat(transform.scale.y);//.mul(0.10147); // Multiplied by scaling factor

            // Perform the vector transformation using Bevy's Transform component
            // let transformed_vector : Mat4 = transform.transform_point(position_vector);
    
            // Push the transformed vector into vertexvec
            // space_vertices.push(transformed_vector.into());
            space_vertices.push(transform.transform_point(position_vector).into());

        }
}

fn extend_line_list_vec(line_vertices: &mut Vec<Vec3>, transform: &Mat4, parent_transform: &Mat4) {

    if line_vertices.is_empty() {
        // First Point, Origin (0,0,0)
        line_vertices.push(Vec3::default());
        // Second Point, from path
        line_vertices.push(transform.transform_point3(Vec3::default()));
    }
    else {
        // Frist Point, fresh calculated from string that was delivered minus the last word -> "finding parent"
        line_vertices.push(parent_transform.transform_point3(Vec3::default()));
        // Second Point, out of given transform
        line_vertices.push(transform.transform_point3(Vec3::default()))
    }
}

    // use just this for linestrip in order of calls, 
fn _extend_line_strip_vec(line_vertices: &mut Vec<Vec3>, transform: &PrimitiveTransform, _cnt: f32, _entry: &DirEntry) {
    line_vertices.push(Mat4::from_cols_array(&transform).transform_point3(Vec3::default()));
}

fn _count_directories(path: &str) -> i32{
    let mut cnt = -1;
    for entry in WalkDir::new(path).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_dir() {
            cnt += 1; 
        }} cnt 
}

// replace rotation with Vec3 when working, only uses Y right now
fn _generate_primitive_transfrom(roation: Vec3, translation: Vec3) -> PrimitiveTransform {
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

