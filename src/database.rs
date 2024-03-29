
use std::collections::btree_map::Iter;
use std::f32::consts::PI;
use std::usize;

use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::math::*;      // Affine3A
use bevy::math::bounding::*;
use bevy::prelude::*;   
use bevy::render::render_asset::RenderAssetUsages;
use bevy::text::YAxisOrientation;
use bevy::utils::RandomState;
use walkdir::{WalkDir, DirEntry};

use std::collections::HashMap;

use crate::generator;

// #[derive(Component, Debug)]
// struct Database {
//     forrest: vec<Tree>,
// }

// impl Database {
//     pub fn new(path: String){

// }

#[derive(Component, Debug)]
pub struct Tree{
    pub path: String,
    pub siblings: Vec<i32>,
    pub branches: Vec<Branch>,
    // pub entity: Vec<Entity>,
    pub bounds: Vec<BoundingSphere>,
    pub hash_map: HashMap<String, usize>
}

impl Tree {
    pub fn new() -> Self {
        Self {
            path : "path".to_string(),
            siblings : Vec::new(),
            branches : Vec::new(),
            // entity : Vec::new(),
            bounds : Vec::new(),
            hash_map: HashMap::new()
        }
    }
    
    pub fn construct(&mut self, path: String) { 

        let maxdepth = 2;
        let mut id_index: usize = 0;
        self.siblings.push(0);
        
        for entry in WalkDir::new(path).max_depth(maxdepth).sort_by(|a,b| a.file_name().cmp(b.file_name())).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() {

                // let ast =                     Branch{ name: entry.path().to_str().unwrap().to_string(), num_of_children:
                //             count_directories(entry.path().to_str().unwrap())} ;
    
                // Would need windows addition anyway
                if entry.path().to_str().unwrap().to_string() == "/".to_string() {
                    self.branches.push( Branch{ 
                        name: "/".to_string(), 
                        num_of_children: count_directories("/"), 
                        children: Vec::new(),
                        transform: Mat4::default(),
                        }
                        );
                    self.hash_map.insert("/".to_string(), id_index);
                    id_index = 1;
                }
                else {
                self.branches.push( Branch{ 
                    name: entry.path().to_str().unwrap().to_string(), 
                    num_of_children: count_directories(entry.path().to_str().unwrap()), 
                    children: Vec::new(),
                    transform: Mat4::default(),
                    }
                    );
                
                    self.hash_map.insert(entry.path().to_str().unwrap().to_string(), id_index);
                }

                // Get parent from branch vector over the index extracted from the hash map by shortening the path string by one directory
                match self.hash_map.get(&get_parent_path(&entry.path().to_str().unwrap().to_string())) {
                    None => println!("No Parent! Path: {:?}, GivenPath: {:?}", &get_parent_path(&entry.path().to_str().unwrap().to_string()),&entry.path().to_str().unwrap().to_string()),
                    Some(index) => {
                        println!("Parentpath: {:?}, Name: {:?}", index, &entry.path().to_str().unwrap().to_string());
                        self.branches.get_mut(*index).unwrap().children.push(id_index);
                    },
                }

                // println!("Values: {:?}", self.hash_map.keys());
                println!("Id_index: {:?}", id_index);

                id_index += 1;

                // Count siblings grouped by depth
                if self.siblings.len() <= entry.depth() {
                    self.siblings.push(1);
                } 
                else {
                    *self.siblings.get_mut(entry.depth()).unwrap() += 1;
                }
            }
        }
    }
    

    pub fn grow(&mut self) -> Mesh {

        let mut line_mesh : Mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
        let mut line_vertices: Vec<Vec3> = vec![];
        let mut pos = Vec3::splat(0.);
        let mut last_pos = Vec3::splat(0.);

        // for (i, ast) in self.branches.clone().into_iter().enumerate() {
        //     pos.x = f32::sin(i as f32 * 0.5);
        //     pos.y = i as f32;
        //     pos.z = f32::cos(i as f32 * 0.5);// + f32::cos(i as f32 * 0.1)*4.;
        //     line_vertices.push(pos);

        //     pos - last_pos;

        //     last_pos = pos;
        // }

        // println!("Heere");
        // for (i, child) in self.branches[0].clone().children.into_iter().enumerate() {
        //     println!("i: {:?} child: {:?}, vec: {:?}", i, child, self.branches.get(child).unwrap().children);
        //     for child in self.branches[child].clone().children.into_iter() {
        //         println!("child: {:?}", child);
        //     }
        // }
        dive(0, &mut self.branches, &mut line_vertices);

        line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, line_vertices);
        line_mesh
    }

    pub fn mesh_nodes(&self) -> Mesh {

        let mut node_mesh : Mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        let mut node_vertices: Vec<[f32; 3]> = vec![];
        let mut node_indices:  Vec<u32> = vec![];

        for (cnt, branch) in self.branches.clone().into_iter().enumerate() {
            generator::extend_space_vec(&mut node_vertices, &mut node_indices, &branch.transform, cnt as f32);

            // println!("Transform: {:?}", branch.transform);
        }

        let space_uvs = vec![[0f32, 0f32]; node_vertices.len()];

        node_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, node_vertices.clone());
        node_mesh.insert_indices(mesh::Indices::U32(node_indices));
        node_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, space_uvs);
        node_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, node_vertices); // Normals are just the vertex positions as we go out from 0,0,0
    
        node_mesh
    }

}

    fn dive(mut index: usize, branches: &mut Vec<Branch>, line_vertices: &mut Vec<Vec3>) -> () {
        if branches.len() > index {
            let mut pos: Vec3 = Vec3::splat(0.);
            let mut last_pos: Vec3 = Vec3::splat(0.);

            // let mut rotation = Vec3::splat(0.5);
            // let mut rotation_quat :Quat = Quat::default();
            let mut translation = Vec3{x:0.,y:1.,z:1.};

            let extending_factor = 20;


            // let branch_transform = branches[index].transform;

            // for child_index in branches[index].children.clone() {
            //     if !(branches.len() < child_index) {

            // Mat4::from_rotation_y(rotation.y) * Mat4::from_translation(translation)
            
            // let branch_transform = branches[child_index].transform;
            let children = branches[index].children.clone();
            let mut inner_child_index: usize = 0;

            // println!("\nCollected Children: {:?}, \n index: {:?} ", branches[index].children, child_index);

            let mut offset = vec3(0., 0., 0.);

            let mut transform = branches[index].transform;// * Mat4::from_translation(translation) ;
            pos = transform.transform_point(Vec3::splat(0.)); 
            last_pos = pos;
            let mut spiral_pos = Vec3::splat(0.);//pos;
            let (vs, q, vt) = transform.to_scale_rotation_translation();
            // transform *= Mat4::from_quat(q);
            // pos = transform.transform_point(pos);

            for i in 0..branches[index].children.len() as i32 * extending_factor { // Number of vertices of branch

                // pos.x = f32::sin(i as f32 * 0.5);
                // pos.y = i as f32;
                // pos.z = f32::cos(i as f32 * 0.5);
                // let mut transform = branches[index].transform;

                // Apart from the first iteration, where transform should be default, all following children should have
                // their own transform when we arrive here
                // transform = branches[index].transform * Mat4::from_rotation_y(i as f32 * 0.01) * Mat4::from_translation(translation);// * Mat4::from_translation(translation) ;

                // transform = branches[index].transform * Mat4::from_translation(translation) * Mat4::from_quat(Quat::from_xyzw(0., i as f32*0.01, 0., 0.));//* branches[index].transform;// * Mat4::from_translation(translation) ;
                // transform = Mat4::from_quat(q);
                // transform = Mat4::from_rotation_y(i as f32 * 0.01) * Mat4::from_translation(translation);// * Mat4::from_quat(q);//* branches[index].transform;// * Mat4::from_translation(translation) ;
                // transform *= Mat4::from_translation(-spiral_pos);
                // transform *= Mat4::from_quat(q);
                
                // transform = Mat4::from_mat3(calc_rotation_matrix(vec3(f32::sin(i as f32 *0.01), 0., f32::cos(i as f32 *0.01)), vec3(-1.,0.,-1.,))) * Mat4::from_translation(translation);// * Mat4::from_quat(q);//* branches[index].transform;// * Mat4::from_translation(translation) ;
                // transform += Mat4::from_translation(translation);// * Mat4::from_quat(q);//* branches[index].transform;// * Mat4::from_translation(translation) ;
                // transform *= branches[index].transform;
                println!("\nRotations \n{:?}\n{:?}\n", Mat4::from_rotation_y(i as f32 * 0.01), Mat4::from_mat3(calc_rotation_matrix(vec3(f32::sin(i as f32 *0.01), 0., f32::cos(i as f32 *0.01)), vec3(0.,0.,-1.,))));
                // pos = transform.transform_point(pos);
                // transform *= Mat4::from_quat(Quat::from_mat4(&branches[index].transform ));
                // pos = transform.transform_point(pos);

                // println!("Trans: {:?}",transform.to_scale_rotation_translation());
                // if i != 0 {
                //     transform *= Mat4::from_quat(q);
                // }      
                // else {
                //     pos = transform.transform_point(pos);
                // }
                if  true {
                    // spiral_pos.x =4.* f32::sin(i as f32 * 0.1);
                    // spiral_pos.y =0.5* i as f32 +1.;
                    // spiral_pos.z =4.* -f32::cos(i as f32 * 0.1);

                    // transform = Mat4::from_quat(q);
                    transform = Mat4::from_quat(q) * Mat4::from_rotation_y(i as f32 * 0.001);// * Mat4::from_scale(vs);// * Mat4::from_translation(translation);// * Mat4::from_quat(q);// * Mat4::from_quat(q);//* branches[index].transform;// * Mat4::from_translation(translation) ;
                    let mut tr:Transform = Transform::from_matrix(transform);
                    tr.translation = translation;
                    transform = tr.compute_matrix();
                    spiral_pos = transform.transform_point3(spiral_pos);
                    // spiral_pos += vt;
                    pos = spiral_pos + vt;
                    // transform = Mat4::from_rotation_y(i as f32 * 0.01) * Mat4::from_translation(Vec3 { x: 0., y: i as f32, z: 1. });//translation);// * Mat4::from_quat(q);//* branches[index].transform;// * Mat4::from_translation(translation) ;
                    // transform += Mat4::from_translation(vt);
                    
                    // transform *= Mat4::from_quat(q);
                    // transform = branches[index].transform;
                    // spiral_pos = transform.transform_point(spiral_pos);
                    // pos = transform.transform_point(spiral_pos);//spiral_pos;
                    // pos
                    // spiral_pos = Vec3::splat(0.);
                    // pos = Transform::from_rotation(q).transform_point(pos);
                    // transform *= Mat4::from_quat(q);
                    // pos = transform.transform_point(pos);
                }
                // if i == 1 {
                //     // pos = transform.transform_point(-spiral_pos);
                //     // transform = transform - Mat4::from_translation(spiral_pos);
                //     // transform *= Mat4::from_quat(q);
                //     // transform = transform + Mat4::from_translation(spiral_pos);

                //     // transform *= Mat4::from_quat(q);

                //     pos = transform.transform_point(spiral_pos);
                //     spiral_pos = pos;
                //     // pos = Transform::from_rotation(q).transform_point(pos);
                // }
                // if i == 2 {
                //     pos = transform.transform_point(pos);
                //     spiral_pos = pos;
                //     // pos = Transform::from_rotation(q).transform_point(pos);
                // }
                // if i==1{
                //     pos = transform.transform_point(spiral_pos);
                //     spiral_pos = pos;
                // }

                else {
                    transform = Mat4::from_rotation_y(i as f32 * 0.01) * Mat4::from_translation(translation);// * Mat4::from_quat(q);//* branches[index].transform;// * Mat4::from_translation(translation) ;

                    pos = transform.transform_point(spiral_pos);
                    spiral_pos = pos;
                    pos = Transform::from_rotation(q).transform_point(pos);
                    pos += vt;
                }

                    // transform *= Mat4::from_quat(q);
                    // pos = transform.transform_point(pos);
                // }


                
                

                // println!("Transform: \n{:?}\nTransform: \n{:?}",branches[index].transform, transform );
                
                if i % extending_factor == extending_factor - 1 {//&& children[inner_child_index] <= children.len(){
                    // pos.x += 10.;
                    let dir = pos - last_pos;
                    // let direction = Direction3d::new(dir).unwrap();    
                    // rotation_quat = Quat::from_rotation_arc_colinear(branches[index].transform, pos);
                    // let mut rts = Transform::from_rotation(Quat::from_rotation_z(0.1));//
                    let mut rts = Transform::from_matrix(transform);// * Transform::from_matrix(branches[index].transform);

                    // rts = rts.with_translation(pos);
                    // rts = rts * rts.looking_to(pos, dir);
                    // rts.rotate_local_axis(pos, 45.);

                    // rts = rts.looking_at(pos*2., dir);
                    // println!("Children: {:?} \nChildIndex: {:?}, \nChild:  {:?}", children,  inner_child_index, children[inner_child_index]);
                    // println!("ChildrenLen: {:?}\ni:  {:?} \n\n", children.len(), i);
                    // rts.scale = Vec3::splat(1.-0.05*inner_child_index as f32);
                    // println!("i: {:?} \niMax: {:?}",i, branches[index].num_of_children * extending_factor);

                    // rts.translation.y -= 1.;
                    // rts = rts.with_translation(pos);

                    // rts = rts.with_rotation(Quat::from_mat4(&Mat4::from_rotation_z(PI/2.)));
                    // rts = rts.with_rotation(Quat::from_mat3(&calc_rotation_matrix(pos ,dir))*Quat::from_mat4(&Mat4::from_rotation_x(PI/2.))*Quat::from_mat3(&calc_rotation_matrix(Vec3 { x: 0., y: 1., z: 0. } ,dir)));
                    // rts = rts.with_rotation(Quat::from_mat3(&calc_rotation_matrix(Vec3 { x: 0., y: 1., z: 1. } ,dir)));
                    let forward = transform.transform_point3(Vec3 { x: 0., y: 1., z: 0. });
                    // rts = rts.with_rotation(Quat::from_mat3(&calc_rotation_matrix(rts.forward().abs(),dir)));

                    let orig_rts = Transform::from_matrix(branches[index].transform);
                    // rts = rts.with_rotation(Quat::from_mat3(&calc_rotation_matrix(rts.local_y().any_orthonormal_vector(),dir.normalize())));
                    // rts = rts.with_translation(-pos);
                    
                    // rts = rts.looking_at(dir, rts.up().any_orthonormal_vector());
                    // rts.look_to(dir, rts.up().any_orthonormal_vector());
                    // rts.look_to(dir, rts.up().any_orthonormal_vector());

                    println!("Pos: {:?} \n last_pos: {:?} \n dir: {:?}", pos, last_pos, dir);
                    // rts = rts.with_rotation(Quat::from_mat3(&calc_rotation_matrix(rts.local_y().normalize(), orig_rts.local_x().normalize())));
                    rts = rts.with_rotation(Quat::from_mat3(&calc_rotation_matrix(rts.up().normalize(), rts.forward().normalize())));

                    rts = rts.with_rotation(Quat::from_mat3(&calc_rotation_matrix(Vec3 { x: 0., y: 0., z: -1. }, dir.normalize())));

                    // rts = rts.with_rotation(orig_rts.rotation);
                    rts.look_at(orig_rts.down().normalize(), rts.local_z().normalize());
                    rts = rts.with_translation(pos);

                    // rts = rts.with_scale(Vec3::splat(0.5));

                    // transform =  Mat4::from_translation(pos) * transform * Mat4::from_quat( Quat::from_mat3(&calc_rotation_matrix(dir ,pos)));
                    
                    branches[children[inner_child_index]].transform = Mat4::from_scale_rotation_translation(rts.scale, rts.rotation, rts.translation); //Mat4::from_translation(pos+offset);////+ Mat4::from_translation(vec3(0., 0., 0.));
                    // offset.x += 10.;

                    // println!("Transform: \n{:?}\nTransform: \n{:?} \n InnerChildIndex: {:?} \n ChildrenLen: {:?} \n Children: {:?}\n", transform, branches[children[inner_child_index]].transform, inner_child_index, children.len(), children);
                    // println!("i: {:?}, imax: {:?}",i, branches[index].children.len() as i32 * extending_factor);
                    // println!("ChildName: {:?}", branches[children[inner_child_index]]);

                    // pos = branches[children[inner_child_index]].transform.transform_point3(pos);
                    inner_child_index += 1;
                
                    // branches[index].transform = Mat4::from_scale_rotation_translation(rts.scale, rts.rotation, rts.translation);//Mat4::from_rotation_translation(rotation_quat, dir); //* Mat4::from_rotation_x(0.1);
                }
                // if i == 0 {
                //     last_pos = vec3(0., 10., 0.);
                // }
                    line_vertices.push(last_pos);
                    line_vertices.push(pos);
                
                    last_pos = pos;
            }
            for child_index in branches[index].children.clone() {
                if !(branches.len() < child_index) {
                    // println!("ChildIndex: {:?}", child_index);
                    dive(child_index, branches, line_vertices);
                }
            }
            }
        // }
        // }
            // dive(depth, branches);
    }

    fn get_parent_path(path: &str) -> String{
        let parent_string: String = match path.rsplit_once("/") {
            Some(cut_path) => {
                if cut_path.0.to_string() == "".to_string(){
                                                "/".to_string()}
                                                else{
                                                cut_path.0.to_string()}
                                                }
            None    => "/".to_string(),
        };
        parent_string
    }

pub fn calc_rotation_matrix(a: Vec3, b: Vec3) -> Mat3 {

    // let a = vec3(0., 0.5, 0.);
    // let b = vec3(1., 0., 0.);

    let v = a.normalize().cross(b.normalize());
    let s = ( v.x.exp2() + v.y.exp2() + v.z.exp2() ).sqrt();
    let c = a.normalize().dot(b.normalize());
    //Axis as rows
    let vx = mat3(vec3(0., -v.z, v.y), vec3(v.z, 0., -v.x), vec3(-v.y, v.x, 0.));
    //Axis as cols
    // let vx = mat3(vec3(0., v.z, -v.y), vec3(-v.z, 0., v.x), vec3(v.y, -v.x, 0.));
    
    let vx2 = dot_product_mat3(vx, vx);
    // let vx2 = vx*vx;
    let rot_mat = Mat3::IDENTITY + vx + vx2 *(1.0 - c);//((1.-c)/s.exp2());
    // println!("RotMat: {:?}", rot_mat);
    rot_mat
}

// pub fn calc_rotation_matrix(a: Vec3, b: Vec3) -> Mat3 {
//     let v = a.normalize().cross(b.normalize());
//     let c = a.normalize().dot(b.normalize());
//     let vx = Mat3::from_cols(
//         vec3(0., -v.z, v.y),
//         vec3(v.z, 0., -v.x),
//         vec3(-v.y, v.x, 0.)
//     );
//     let vx2 = vx * vx;
//     let rot_mat = Mat3::IDENTITY + vx + vx2 * (1.0 - c);
//     rot_mat
// }

fn dot_product_mat3(mat1: Mat3, mat2: Mat3) -> Mat3 {
    // let mut dot_product = 0.0;
    let mut result = Mat3::IDENTITY;

    // x axis interpreted as row
    result.x_axis = vec3(mat1.row(0).dot(mat2.col(0)), mat1.row(0).dot(mat2.col(1)), mat1.row(0).dot(mat2.col(2)));
    result.y_axis = vec3(mat1.row(1).dot(mat2.col(0)), mat1.row(1).dot(mat2.col(1)), mat1.row(1).dot(mat2.col(2)));
    result.z_axis = vec3(mat1.row(2).dot(mat2.col(0)), mat1.row(2).dot(mat2.col(1)), mat1.row(2).dot(mat2.col(2)));
    
    // axis as col
    // result.x_axis = vec3(mat1.row(0).dot(mat2.col(0)), 
    // mat1.row(1).dot(mat2.col(0)),
    // mat1.row(2).dot(mat2.col(0)));

    // result.y_axis =vec3(mat1.row(0).dot(mat2.col(1)),
    // mat1.row(1).dot(mat2.col(1)),
    // mat1.row(2).dot(mat2.col(1)));

    // result.z_axis = vec3(mat1.row(0).dot(mat2.col(2)),
    // mat1.row(1).dot(mat2.col(2)),
    // mat1.row(2).dot(mat2.col(2)));

    // result.x_axis = vec3(1.,2.,3.,);
    // result.y_axis = vec3(4.,5.,6.,);
    // result.z_axis = vec3(7.,8.,9.,);

    println!("Mat3: {:?}", result);

    result 
}
    
pub fn count_directories(path: &str) -> i32
{
    let mut cnt = -1;
    for entry in WalkDir::new(path).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_dir() {
            cnt += 1; 
        }}
        // println!("Count: {:?}",cnt);
         cnt 
}

#[derive(Component, Debug, Clone)]
pub struct Branch {
    // id: Entity,
    pub name: String,
    pub num_of_children: i32,
    pub children: Vec<usize>,

    pub transform: Mat4,

    // position: Transform, // how we got here
    // lind: String, // holds the information to construct geometric branch
}

impl Branch {
    pub fn new() -> Self {
        Self {
            name : "".to_string(),
            num_of_children : 0,
            children: Vec::new(),
            transform: Mat4::default(),
        }
    }
}

// Fill entity list of branches through string:cutting like before? /sys/bus -> add to /sys/
// Number of direct siblings is found in parent -> children

