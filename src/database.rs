
use std::collections::btree_map::Iter;
use std::usize;

use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::math::*;      // Affine3A
use bevy::math::bounding::*;
use bevy::prelude::*;   
use bevy::render::render_asset::RenderAssetUsages;
use bevy::utils::RandomState;
use walkdir::{WalkDir, DirEntry};

use std::collections::HashMap;

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
        println!("Heere");

        for (i, child) in self.branches[0].clone().children.into_iter().enumerate() {
            println!("i: {:?} child: {:?}, vec: {:?}", i, child, self.branches.get(child).unwrap().children);
            for child in self.branches[child].clone().children.into_iter() {
                println!("child: {:?}", child);
            }
        }
        dive(0, &mut self.branches, &mut line_vertices);

        line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, line_vertices);
        line_mesh
    }
}

    fn dive(mut index: usize, branches: &mut Vec<Branch>, line_vertices: &mut Vec<Vec3>) -> () {
        if branches.len() > index {
            let mut pos: Vec3 = Vec3::splat(0.);
            let mut last_pos: Vec3 = Vec3::splat(0.);

            let mut rotation = Vec3::splat(1.);
            let mut rotation_quat :Quat = Quat::default();
            let mut translation = Vec3::splat(1.);

            let extening_factor = 20;

            pos = branches[index].transform.transform_point(pos);

            for child_index in branches[index].children.clone() {
                println!("Collected Children: {:?} ", branches[index].children);

                // Mat4::from_rotation_y(rotation.y) * Mat4::from_translation(translation)

                // let mut transform = branches[index].transform;

                for i in 0..branches[index].num_of_children * extening_factor { // Number of vertices of branch
                    // pos.x = f32::sin(i as f32 * 0.5);
                    // pos.y = i as f32;
                    // pos.z = f32::cos(i as f32 * 0.5);
                    let mut transform = branches[index].transform;

                    transform = transform * Mat4::from_rotation_y(i as f32 * 0.01) * Mat4::from_translation(translation);
                    pos = transform.transform_point(pos);
                   
                    if i % extening_factor == 0 {
                        // pos.x += 10.;
                        let dir = pos - last_pos;
                        // let direction = Direction3d::new(dir).unwrap();    
                        // rotation_quat = Quat::from_rotation_arc_colinear(branches[index].transform, pos);
                        let mut rts = Transform::from_translation(Vec3{x:0.,y:0.,z:0.});
                        rts = rts.with_translation(pos);
                        rts = rts.looking_to(pos, dir);
                        // rts = rts.looking_at(pos*2., dir);
                        branches[index].transform = Mat4::from_scale_rotation_translation(rts.scale, rts.rotation, rts.translation);//Mat4::from_rotation_translation(rotation_quat, dir); //* Mat4::from_rotation_x(0.1);
                    }
                   
                    line_vertices.push(last_pos);
                    line_vertices.push(pos);
                    last_pos = pos;
                }
                dive(child_index, branches, line_vertices);
            }
        }
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

