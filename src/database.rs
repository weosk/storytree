
use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::math::*;      // Affine3A
use bevy::math::bounding::*;
use bevy::prelude::*;   
use bevy::render::render_asset::RenderAssetUsages;
use bevy::utils::RandomState;
use walkdir::{WalkDir, DirEntry};



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
    pub branch: Vec<Branch>,
    pub entity: Vec<Entity>,
    pub bounds: Vec<BoundingSphere>,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            path : "path".to_string(),
            siblings : Vec::new(),
            branch : Vec::new(),
            entity : Vec::new(),
            bounds : Vec::new(),
        }
    }
    
    pub fn construct(&mut self, path: String){
        let maxdepth = 100;
        self.siblings.push(0);
        for entry in WalkDir::new(path).max_depth(maxdepth).sort_by(|a,b| a.file_name().cmp(b.file_name())).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() {

                // let ast =                     Branch{ name: entry.path().to_str().unwrap().to_string(), num_of_children:
                //             count_directories(entry.path().to_str().unwrap())} ;

                self.branch.push( Branch{ 
                    name: entry.path().to_str().unwrap().to_string(), 
                    num_of_children: count_directories(entry.path().to_str().unwrap())} );
                }
                // println!("Siblign: {:?}, {:?}", self.siblings.len(), entry.depth());
                if self.siblings.len() <= entry.depth() {
                    self.siblings.push(1);
                } 
                else {
                    // println!("Siblign: {:?}", entry.path().to_str().unwrap().to_string());
                    *self.siblings.get_mut(entry.depth()).unwrap() += 1;
                }
            }
        }


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

#[derive(Component, Debug)]
pub struct Branch {
    // id: Entity,
    name: String,
    num_of_children: i32,
    // children: vec<Entity>,
    // position: Transform, // how we got here
    // lind: String, // holds the information to construct geometric branch
}

impl Branch {
    pub fn new() -> Self {
        Self {
            name : "".to_string(),
            num_of_children : 0,
        }
    }
}