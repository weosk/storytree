
use std::usize;
use std::f32::consts::{E, PI};
use std::collections::HashMap;

use bevy::render::mesh::{self, PrimitiveTopology};
use bevy::math::*;      // Affine3A
use bevy::math::bounding::*;
use bevy::prelude::*;   
use bevy::render::render_asset::RenderAssetUsages;
use walkdir::WalkDir;

use crate::generator;

#[derive(Component, Debug, Clone)]
pub struct Branch {
    // id: Entity,
    pub name: String,
    pub num_of_children: i32,    // only direkt children
    pub num_of_all_children: i32,// recursivly all
    pub children: Vec<usize>,
    pub parent: usize,

    pub transform: Transform,
    pub depth: usize,

    // position: Transform, // how we got here
    // lind: String, // holds the information to construct geometric branch
}
impl Branch {
    pub fn new() -> Self {
        Self {
            name : "".to_string(),
            num_of_children : 0,
            num_of_all_children: 0,
            parent : 0,
            children: Vec::new(),
            transform: Transform::default(),//Mat4::default(),
            depth : 0,
        }
    }
}

// Fill entity list of branches through string:cutting like before? /sys/bus -> add to /sys/
// Number of direct siblings is found in parent -> children




#[derive(Component, Debug, Resource, Clone)]
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

        let maxdepth = 30;
        let mut id_index: usize = 0;
        self.siblings.push(0);
        
        for entry in WalkDir::new(path).max_depth(maxdepth).sort_by(|a,b| a.file_name().cmp(b.file_name())).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() {

                // let ast = Branch{ name: entry.path().to_str().unwrap().to_string(), num_of_children: count_directories(entry.path().to_str().unwrap())} ;
                // Would need windows addition anyway
                if entry.path().to_str().unwrap().to_string() == "/".to_string() {
                    self.branches.push( Branch{ 
                        name: "/".to_string(), 
                        num_of_children: count_directories("/"), 
                        num_of_all_children: 0, // initialised to zero, 
                        children: Vec::new(),
                        parent: 0, // root is it's  own parent
                        transform: Transform::default(),
                        depth: entry.depth(),
                        }
                        );
                    self.hash_map.insert("/".to_string(), id_index);
                    // id_index = 1;
                }
                else {
                    self.branches.push( Branch{ 
                                        name: entry.path().to_str().unwrap().to_string(), 
                                        num_of_children: count_directories(entry.path().to_str().unwrap()), 
                                        num_of_all_children: 0, // initialised to zero, 
                                        children: Vec::new(),
                                        parent: 0,
                                        transform: Transform::default(),//Mat4::default(),
                                        depth: entry.depth(),
                                        }
                        );
                    self.hash_map.insert(entry.path().to_str().unwrap().to_string(), id_index);
                }
                // if id_index < 10 {
                //     info!("\nId_index: {:?} \nHashmap: {:?} \nBranches: {:?}",id_index, self.hash_map, self.branches);
                // }
                // Get parent from branch vector over the index extracted from the hash map by shortening the path string by one directory
                if id_index > 0 {
                    match self.hash_map.get(&get_parent_path(&entry.path().to_str().unwrap().to_string())) {
                        None => println!("No Parent! Path: {:?}, GivenPath: {:?}", &get_parent_path(&entry.path().to_str().unwrap().to_string()),&entry.path().to_str().unwrap().to_string()),
                        Some(index) => {
                            // println!("Parentpath: {:?}, Name: {:?}", index, &entry.path().to_str().unwrap().to_string());
                            self.branches.get_mut(*index).unwrap().children.push(id_index);

                            // Assign parent to child
                            // info!("Branch: {:?}",self.branches.last());
                            // info!("index: {:?}",id_index);
                            // info!("parentindex: {:?}",*index);

                            // if *index == id_index{
                            //     panic!("Panik!");
                            // }

                            self.branches.last_mut().unwrap().parent = *index;

                            // if *index == 0 {
                            //     info!("\nIIIndex: {:?} \nIndexParent: {:?} \nChildren: {:?}",0, self.branches[0].parent, self.branches[0].children);
                            //     info!("Path: {:?} Entry: {:?} \nParentpath: {:?}", *&entry.path().to_str().unwrap().to_string(), entry, &get_parent_path(&entry.path().to_str().unwrap().to_string()));
                            //     println!("HashValues: {:?}", self.hash_map.get(&get_parent_path(&entry.path().to_str().unwrap().to_string())));
                            // }
                        }
                    }
                }

                // println!("Id_index: {:?}", id_index);

                // Fill bounds with spheres
                self.bounds.push(BoundingSphere { center: Vec3::splat(0.), sphere: primitives::Sphere{ radius: 1.0 }});
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

    // Wrapper method for the recursive dive to provide the mesh
    pub fn grow(&mut self, name: &mut str, param_set: (f32,f32,f32,f32)) -> Mesh {

        let mut line_mesh : Mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default());
        let mut line_vertices: Vec<Vec3> = vec![];

        // info!("Root Children1:{:?}",self.branches[0].children);

        // dive to count all the subfolders
        info!("All Subfolders: {:?}",dive_to_count(0, &mut self.branches)); 

        // dive to sort
        dive_to_sort(0,&mut self.branches);

        // dive to construct
        // dive(name,0, &mut self.branches, &mut line_vertices);
        param_dive(name,0, &mut self.branches, &mut line_vertices,  param_set);


//         for i in 0..5 {
//             info!("i: {:?} Children: {:?}",i ,self.branches[i].children);
//             info!("i: {:?} Parent: {:?}",i ,self.branches[i].parent);
//             info!("i: {:?} NumOfChildren: {:?}",i ,self.branches[i].num_of_children);
//             info!("i: {:?} NumOfAllChildren: {:?} : {:?}",i ,self.branches[i].num_of_all_children, self.branches[i].transform);
//             info!("Transform0: {:?}",self.branches[i].transform);
//         }

        // info!("Number of branches: {}", self.branches.len());

        line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, line_vertices);
        line_mesh
    }

    // Provides bounds & dodecamesh
    pub fn mesh_nodes(&mut self) -> Mesh {

        let mut node_mesh : Mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        let mut node_vertices: Vec<[f32; 3]> = vec![];
        let mut node_indices:  Vec<u32> = vec![];

        for (cnt, branch) in self.branches.clone().into_iter().enumerate() {
            generator::extend_space_vec(&mut node_vertices, 
                                        &mut node_indices, 
                                        &branch.transform.compute_matrix(), cnt as f32);

            // Populate bounds
            self.bounds[cnt].sphere = primitives::Sphere{ radius: branch.transform.scale.y };
            self.bounds[cnt].center = branch.transform.translation;
        }

        let space_uvs = vec![[0f32, 0f32]; node_vertices.len()];

        node_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, node_vertices.clone());
        node_mesh.insert_indices(mesh::Indices::U32(node_indices));
        node_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, space_uvs);
        node_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, node_vertices); 
        // Normals are just the vertex positions as we go out from 0,0,0
    
        node_mesh
    }

    // println!("Bounds: {:?}, {:?}", self.bounds[cnt].center, self.bounds[cnt].sphere);
    // println!("Transform: {:?}", branch.transform);
}

// Returns number of subfolder count
fn dive_to_count(index: usize, branches: &mut Vec<Branch>) -> i32 { 
    // info!("IndexCount: {:?}", index);
    if branches.len() > index {
        for child_index in branches[index].children.clone() {
            if branches.len() > child_index {
                // info!("IndexCount: {:?}, ChildIndex: {:?}, Children: {:?}", index, child_index, branches[index].children);
                
                branches[index].num_of_all_children += dive_to_count(child_index, branches);
            }
        }
        return branches[index].num_of_all_children +1;
    }
    error!("branches.len() > index: {:?} ... {:?}",branches.len(),index);
    return 0;
}

// Sorts children
fn dive_to_sort(index: usize, branches: &mut Vec<Branch>) { 

    if branches.len() > index && branches[index].children.len() != 0 {
        let mut sort_vec: Vec<(usize,i32)> = vec![];

        for i in 0..branches[index].children.len() {
            if branches[index].children[i] < branches.len() {

                if index < 3 {
                    // info!("--- Index: {:?} : {:?}", index, branches[branches[index].children[i]].num_of_all_children)
                }
                sort_vec.push((branches[index].children[i], branches[branches[index].children[i]].num_of_all_children)); // Child indizes der Reihe nach
            }
            else {
                error!("Descendantindex out of bounds!");
            }
        }
        // if index < 2 {
        //     info!("SortVec: {:?} \n", sort_vec);
        // }
        if sort_vec.len() > 0 {
            // info!("1 SortVec: {:?}",sort_vec);
            sort_vec.sort_by_key(|k| k.1);
            // Standart is highest value on last position
            // sort_vec.reverse();

            for i in 0..sort_vec.len() {
                branches[index].children[i] = sort_vec[i].0;
            }
        }
        for child_index in branches[index].children.clone() {
            if branches.len() > child_index {
                dive_to_sort(child_index, branches);
            }
        }
    }
}


    fn param_dive(name:&mut str, index: usize, branches: &mut Vec<Branch>, line_vertices: &mut Vec<Vec3>,param_set: (f32,f32,f32,f32)) -> () {
        if branches.len() > index {
            
            let mut extending_factor = 40;//0 - branches[index].depth as i32 * 10; //40;//20;//branches[index].children;//20;
            let children = branches[index].children.clone();
            let mut inner_child_index: usize = 0;
    
            let transform = branches[index].transform;
            let mut pos = transform.translation;
            let mut last_pos = pos;
    
            let mut spiral_pos = Vec3::splat(0.);
            // let mut spiral_pos = vec3(2000., 0., 0.);
    
            let mut spiral_transform = Transform::default();
    
            let scale = param_set.0 * param_set.1.powf(branches[index].depth as f32);
            
            if index == 0 {
                branches[index].transform.scale = Vec3::splat(scale);
                extending_factor = 1000;
            }

            let vertex_iteration = (branches[index].children.len() as i32 * extending_factor) - 0;

            for i in 0..vertex_iteration { // Number of vertices of branch

                // spiral_transform.translation.x =scale*0.001*i as f32 * (i as f32 * PI/16.).cos() + scale* 10.*(1.-1.*E.powf(-0.0001*i as f32)) * 1.*(1./64.*i as f32 * PI/16.).cos();
                // spiral_transform.translation.y =scale* 0.5;//0.5;//0.2;
                // spiral_transform.translation.z =scale*0.001*i as f32 * (i as f32 * PI/16.).sin() + scale* 10.*(1.-1.*E.powf(-0.0001*i as f32)) * 1.*(1./64.*i as f32 * PI/16.).sin();

                
                // spiral_transform.translation.x = scale * (param_set.2 * (i as f32 * PI/16. * param_set.3).cos() + param_set.0 * i as f32);// + scale* 10.*(1.-1.*E.powf(-0.0001*i as f32)) * 1.*(1./64.*i as f32 * PI/16.).cos();
                // spiral_transform.translation.y = scale * param_set.2 * i as f32 *  0.3;//param_set.0 + scale* 1.;
                // spiral_transform.translation.z = scale * (param_set.2 * (i as f32 * PI/16. * param_set.3).sin() + param_set.0 * i as f32);// + scale* 10.*(1.-1.*E.powf(-0.0001*i as f32)) * 1.*(1./64.*i as f32 * PI/16.).sin();

                spiral_transform.translation.x =scale* ( 0.001*i as f32 * (i as f32 * PI/16. + param_set.3).cos() + scale* 10.*(1.-1.*E.powf(-0.0001*i as f32)) * 1.*(1./64.*i as f32 * PI/16.).cos());
                spiral_transform.translation.y =0.01 + branches[index].depth as f32 * 0.1 * scale;//(branches[index].depth as f32 *0.01);// + 0.1 * i as f32) * scale;//param_set.3*scale* 1.;//0.5;//0.2;
                spiral_transform.translation.z =scale* ( 0.001*i as f32 * (i as f32 * PI/16.).sin() + scale* 10.*(1.-1.*E.powf(-0.0001*i as f32)) * 1.*(1./64.*i as f32 * PI/16.).sin());
           
                // Spiral Backup
                // spiral_transform.translation.x =scale* ( 0.001*i as f32 * (i as f32 * PI/16.).cos() + scale* 10.*(1.-1.*E.powf(-0.0001*i as f32)) * 1.*(1./64.*i as f32 * PI/16.).cos());
                // spiral_transform.translation.y =scale * 1.;//param_set.3*scale* 1.;//0.5;//0.2;
                // spiral_transform.translation.z =scale* ( 0.001*i as f32 * (i as f32 * PI/16.).sin() + scale* 10.*(1.-1.*E.powf(-0.0001*i as f32)) * 1.*(1./64.*i as f32 * PI/16.).sin());
           
                // Spiraling Up
                spiral_pos = spiral_transform.transform_point(spiral_pos);
                // Rotate into formerly given direction
                pos = Transform::from_rotation(transform.rotation).transform_point(spiral_pos);
                // Translate to formerly given position
                pos = Transform::from_translation(transform.translation).transform_point(pos);


                if i % extending_factor == extending_factor - 1 {
                    if branches.len() > children[inner_child_index] { // To prevent len == index for /
                        let dir = pos - last_pos;
                        
                        let mut rts = spiral_transform;

                        if children[inner_child_index] == *children.last().unwrap() {
                            rts = rts.with_rotation(branches[branches[index].parent].transform.rotation);
                            rts = rts.with_translation(pos);
                            rts = rts.with_scale(Vec3::splat(scale));

                        }
                        else {
                            // rts.look_to(  pos.normalize().any_orthogonal_vector(), pos.normalize());// * Vec3 {x: 1., y: 0., z: 1. });
                            // rts.look_to(  dir.normalize(), Vec3 {x: 0., y: 1., z: 0. } + pos.normalize());// * Vec3 {x: 1., y: 0., z: 1. });
                            // rts.look_to(  dir.normalize().any_orthonormal_vector(), dir.normalize() + Vec3 {x: 0., y: 1., z: 0. } );// Vec3 {x: 0., y: 1., z: 0. });// * Vec3 {x: 1., y: 0., z: 1. });
                            rts.look_to(  dir.normalize(), Vec3 {x: 0., y: 1., z: 0. } );// * Vec3 {x: 1., y: 0., z: 1. });

                            rts = rts.with_translation(pos);
                            rts = rts.with_scale(Vec3::splat(scale));
                        }    
                        branches[children[inner_child_index]].transform = rts;
                        inner_child_index += 1;
                    }
                }
                    line_vertices.push(last_pos);
                    line_vertices.push(pos);
                    last_pos = pos;
            }

            // param_set.0 -= 0.1;
            // param_set.2 += 0.1;

            for child_index in branches[index].children.clone() {
                if !(branches.len() < child_index) {
                    param_dive(name, child_index, branches, line_vertices, param_set);
                }
            }
        }
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

pub fn _calc_rotation_matrix(a: Vec3, b: Vec3) -> Mat3 {

    // let a = vec3(0., 0.5, 0.);
    // let b = vec3(1., 0., 0.);

    let v = a.normalize().cross(b.normalize());
    let _s = ( v.x.exp2() + v.y.exp2() + v.z.exp2() ).sqrt();
    let c = a.normalize().dot(b.normalize());
    //Axis as rows
    let vx = mat3(vec3(0., -v.z, v.y), vec3(v.z, 0., -v.x), vec3(-v.y, v.x, 0.));
    //Axis as cols
    // let vx = mat3(vec3(0., v.z, -v.y), vec3(-v.z, 0., v.x), vec3(v.y, -v.x, 0.));
    
    let vx2 = _dot_product_mat3(vx, vx);
    // let vx2 = vx*vx;
    let rot_mat = Mat3::IDENTITY + vx + vx2 *(1.0 - c);//((1.-c)/s.exp2());
    // println!("RotMat: {:?}", rot_mat);
    rot_mat
}

fn _dot_product_mat3(mat1: Mat3, mat2: Mat3) -> Mat3 {
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



///
/// Old Dive: 
/// 




// Standart swirl
// spiral_transform.rotate_y(PI/128.);
// spiral_transform.rotate_y(PI/16.);

// Plantlike
// spiral_transform.rotate_y(4.*PI/(index as f32+0.5) as f32);

// Spiral swirl
// spiral_transform.rotate_y();

// ProvenTrees
// extending_factor = 40;
// spiral_transform.translation.y = 1. * scale;
// spiral_transform.translation.z = 0.333 * scale;

// Fixed variant for small data:
// spiral_transform.rotate_y(PI/16.);

// Variable variant for big data:
// spiral_transform.rotate_y(4.*PI/(index as f32+0.5) as f32);

// Parameterset: 
// A tupel list that is given by reference for the whole dive, containing switches and concrete parameters like
// Extending Faktor
// Scale                                // Both as functions?
// SpiralTransform Parameter: a few...
// Orientation Mode

fn _dive(name:&mut str, index: usize, branches: &mut Vec<Branch>, line_vertices: &mut Vec<Vec3>) -> () {

    // if index <= 100 {
    //     info!("\nIndex : {:?}\nParent: {:?}\nSiblings: {:?}",index, branches[index].parent, branches[branches[index].parent].children);   
    // }
    if branches.len() > index {
        let mut pos: Vec3 = Vec3::splat(0.);
        let mut last_pos: Vec3 = Vec3::splat(0.);
        let mut extending_factor = 40;//20;//branches[index].children;//20;
        let children = branches[index].children.clone();
        let mut inner_child_index: usize = 0;

        let mut transform = branches[index].transform;
        pos = transform.translation;//transform.transform_point(Vec3::splat(0.)); 
        last_pos = pos;

        let mut spiral_pos = Vec3::splat(0.);
        // let mut spiral_pos = vec3(2000., 0., 0.);

        let mut spiral_transform = Transform::default();

        let mut scale = 1.;//0.1*(10. - branches[index].depth as f32 * 3.);

        if true {
            extending_factor = branches[index].num_of_all_children+1;//70;//10 *scale as i32;
            scale = 0.5;//2.;//1.;//10. -0.5 * branches[index].depth as f32;
            if extending_factor > 40 {
                extending_factor = 40;
            }
            if index == 0 {
                extending_factor = 40;//1000;
            }
        }
        else { 
            // spiral_transform.translation.y = 0.5 * scale;//* scale;
            // spiral_transform.translation.z = 0.333 * scale * 10.;// * scale;
            // spiral_transform.rotate_y(PI/16.);            // spiral_transform.rotate_y(PI/16.);
        }

        // let scale = Vec3::splat(scale);
        let mut z = 0;
        let mut circle_cnt = 0;

        let vertex_iteration = (branches[index].children.len() as i32 * extending_factor) - 0;

        let mut first_dir_vec = Vec3::default();

        for mut i in 0..vertex_iteration { // Number of vertices of branch
            
            if i == 7 {
                first_dir_vec = pos - last_pos;
            }
            // if index == 0 {
            //     spiral_transform.translation.x =0.7*scale.x *  i as f32 *(i as f32 * PI/27.).cos(); //8.*
            //     spiral_transform.translation.y =1.*scale.y *(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
            //     spiral_transform.translation.z =0.7*scale.z *  i as f32  *(i as f32 * PI/27.).sin();//8.*
            //     // extending_factor = 100;
            // } 
            // else if index == *branches[branches[index].parent].children.last().unwrap() {
            //     // info!("DepthLast: {:?} Index: {:?}", branches[index].depth, index);

            //     let scale = Vec3::splat(0.1);
            //     spiral_transform.translation.x =1.*scale.x *  i as f32 *(i as f32 * PI/16.).cos(); //8.*
            //     spiral_transform.translation.y =branches[index].depth as f32*1.+1.*(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
            //     spiral_transform.translation.z =-0.1*i as f32+1.*scale.z   *(i as f32 * PI/16.).sin();//8.*
            //     // extending_factor = 100;
            // }  
            // else {
            //     spiral_transform.translation.x =0.1*i as f32 *scale.x *(i as f32 * PI/16.).cos(); //8.*
            //     spiral_transform.translation.y =0.2*scale.y *(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
            //     spiral_transform.translation.z =-0.11*i as f32 + 0.5*scale.z *(i as f32 * PI/16.).sin();//8.*
            // }


            if false {//index  == 0 {
            //     spiral_transform.translation.x =0.7*scale.x *  i as f32 *(i as f32 * PI/27.).cos(); //8.*

            //     if false{// i < extending_factor {
            //         spiral_transform.translation.y =40.;//4. *scale.y * (1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
            //     }
            //     else {
            //         spiral_transform.translation.y =2. *scale.y * (0.5-E.powf(-1.*i as f32));//i as f32 ;//8.*
            //     }
            //     spiral_transform.translation.z =0.7*scale.z *  i as f32  *(i as f32 * PI/27.).sin();//8.*
            //     // extending_factor = 100;

                spiral_transform.translation.x =0.7 *  i as f32 *(i as f32 * PI/27.).cos(); //8.*
                spiral_transform.translation.y =2.  * (0.5-E.powf(-1.*i as f32));//i as f32 ;//8.*
                spiral_transform.translation.z =0.7 *  i as f32  *(i as f32 * PI/27.).sin();//8.*

                // spiral_transform.translation.x = (i as f32 * PI/16.).cos() + 1.*(1./32.*i as f32 * PI/16.).cos();
                // spiral_transform.translation.y = 0.5;
                // spiral_transform.translation.z = (i as f32 * PI/16.).sin() + 1.*(1./32.*i as f32 * PI/16.).sin();
            } 
            else if false {// index == *branches[branches[index].parent].children.last().unwrap() {
                // info!("DepthLast: {:?} Index: {:?}", branches[index].depth, index);

                // let scale = Vec3::splat(0.1);
                // spiral_transform.translation.x =1.*scale.x *  i as f32 *(i as f32 * PI/16.).cos(); //8.*
                // spiral_transform.translation.y =3. + branches[index].depth as f32*1.+1.*(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
                // spiral_transform.translation.z =-0.1*i as f32+1.*scale.z   *(i as f32 * PI/16.).sin();//8.*
                // // extending_factor = 100;

                // let scale = Vec3::splat(0.1);
                // spiral_transform.translation.x =1.*scale.x *  i as f32 *(i as f32 * PI/16.).cos(); //8.*
                // spiral_transform.translation.y =3. + branches[index].depth as f32*1.+1.*(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
                // spiral_transform.translation.z =-0.1*i as f32+1.*scale.z   *(i as f32 * PI/16.).sin();//8.*

                // Growing Sprial Funnel
                spiral_transform.translation.x = 40.*(1.-1.*E.powf(-0.0001*i as f32)) * (i as f32 * PI/12.).cos() //- 1.*(1./128.*circle_cnt as f32 * PI/16.).cos();    
                ;//+ 400.*(1.-1.*E.powf(-0.001*i as f32)) * (i as f32 * PI/16.).cos();
                spiral_transform.translation.y = 0.2; //10.*(1.-1.*E.powf(-0.0001*i as f32)) * (i as f32 * PI/16.).sin();
                spiral_transform.translation.z = 40.*(1.-1.*E.powf(-0.0001*i as f32)) * (i as f32 * PI/12.).sin() //+ 1.*(1./128.*circle_cnt as f32 * PI/16.-PI/2.).sin();
                ;//+400.*(1.-1.*E.powf(-0.001*i as f32)) * (i as f32 * PI/16.).sin();
                // (0.5-E.powf(-1.*i as f32))

                     }  
            else {
                // spiral_transform.translation.x =            0.01*scale.x *(i as f32 * PI/16.).cos(); //8.*
                // spiral_transform.translation.y =0.001*scale.y *(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
                // spiral_transform.translation.z =-0.001*i as f32 + 0.5*scale.z *(i as f32 * PI/16.).sin();//8.*

                // spiral_transform.translation.x =            0.01*scale.x *(i as f32 * PI/16.).cos(); //8.*
                // spiral_transform.translation.y =0.001*scale.y *(1.-E.powf(-1.*i as f32)) + 10.;//i as f32 ;//8.*

                // spiral_transform.translation.z =-0.001*i as f32 + 0.5*scale.z *(i as f32 * PI/16.).sin();//8.*

                // if i % 1 == 0 {
                //     circle_cnt += 1;
                // }

                // spiral_transform.translation.x = - 15.*(1.-E.powf(-1.*i as f32)) * (i as f32 * PI/16.).cos(); //- 1.*(1./128.*circle_cnt as f32 * PI/16.).cos();
                // spiral_transform.translation.y = 0.2;
                // spiral_transform.translation.z = - 15.*(1.-E.powf(-1.*i as f32)) * (i as f32 * PI/16.).sin(); //+ 1.*(1./128.*circle_cnt as f32 * PI/16.-PI/2.).sin();
                
                // info!("I: {:}", i);
                // if true {//i < 1000 {    
                //     spiral_transform.translation.x = 0.01*i as f32 * (i as f32 * PI/16.).cos(); //- 1.*(1./128.*circle_cnt as f32 * PI/16.).cos();    
                //     spiral_transform.translation.y = 0.2;
                //     spiral_transform.translation.z = 0.01*i as f32 * (i as f32 * PI/16.).sin(); //+ 1.*(1./128.*circle_cnt as f32 * PI/16.-PI/2.).sin();
                // }
                // else {
                //     spiral_transform.translation.x = 10. * (i as f32 * PI/16.).cos(); //- 1.*(1./128.*circle_cnt as f32 * PI/16.).cos();    
                //     spiral_transform.translation.y = 0.2;
                //     spiral_transform.translation.z = 10. * (i as f32 * PI/16.).sin(); //+ 1.*(1./128.*circle_cnt as f32 * PI/16.-PI/2.).sin(); 
                // }

                // Wingling Big Spiral
            //     }
            //

        }
                    let save_i = i;
                    if children[inner_child_index] == *children.last().unwrap() {
                        // i = branches[children[inner_child_index]].parent\
                        // if branches[branches[index].parent].num_of_children > 40 {
                        //     i += branches[branches[index].parent].num_of_children * 40;
                        // }
                        // else  {
                        //     i += branches[branches[index].parent].num_of_children * branches[branches[index].parent].num_of_all_children;
                        // }
                    }
                    if true {
                        // scale = 10. * branches[index].num_of_all_children as f32 / branches[branches[index].parent].num_of_all_children as f32;
                        // if index == *branches[branches[index].parent].children.last().unwrap() || index == 0{
                        // spiral_transform.translation.x =scale* 0.001*i as f32 + (i as f32 * PI/16.).cos() + 10.*(1.-1.*E.powf(-0.0001*i as f32)) * 1.*(1./64.*i as f32 * PI/16.).cos();
                        // spiral_transform.translation.y =scale* 1.;//0.2;
                        // spiral_transform.translation.z =scale* 0.001*i as f32 + (i as f32 * PI/16.).sin() + 10.*(1.-1.*E.powf(-0.0001*i as f32)) * 1.*(1./64.*i as f32 * PI/16.).sin();

                        scale = 1.;
                        spiral_transform.translation.x =scale*0.001*i as f32 * (i as f32 * PI/16.).cos() + scale* 10.*(1.-1.*E.powf(-0.0001*i as f32)) * 1.*(1./64.*i as f32 * PI/16.).cos();
                        spiral_transform.translation.y =scale* 0.5;//0.5;//0.2;
                        spiral_transform.translation.z =scale*0.001*i as f32 * (i as f32 * PI/16.).sin() + scale* 10.*(1.-1.*E.powf(-0.0001*i as f32)) * 1.*(1./64.*i as f32 * PI/16.).sin();
                   
                    }
                    else {
                        // spiral_transform.translation.x =1.*scale.x *  i as f32 *(i as f32 * PI/16.).cos(); //8.*
                        // spiral_transform.translation.y =3. + branches[index].depth as f32*1.+1.*(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
                        // spiral_transform.translation.z =-0.1*i as f32+1.*scale.z   *(i as f32 * PI/16.).sin();//8.*
                        spiral_transform.translation.x = 0.5 * i as f32 *(i as f32 * PI/16.).cos(); //8.*
                        spiral_transform.translation.y = 3. + branches[index].depth as f32*1.+1.*(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
                        spiral_transform.translation.z = -0.1*i as f32+ (i as f32 * PI/16.).sin();//8.*
                    }
                    if children[inner_child_index] == *children.last().unwrap() {
                        i = save_i;
                    }
            // else {

            // }

            // spiral_transform.translation.x = (i as f32 * PI/16.).cos() + 1.*(1./32.*i as f32 * PI/16.).cos();
            // spiral_transform.translation.y = 0.5;
            // spiral_transform.translation.z = (i as f32 * PI/16.).sin() + 1.*(1./32.*i as f32 * PI/16.).sin();

            // if i < vertex_iteration/2 as i32 {
            //     spiral_transform.translation.x =2. * scale.x *(i as f32 * PI/27.).cos(); //8.*
            //     spiral_transform.translation.y =3. * scale.y *(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
            //     spiral_transform.translation.z =2. * scale.z  *(i as f32 * PI/27.).sin();//8.*



            //     // spiral_transform.translation.x =0.7*scale.x *  i as f32 *(i as f32 * PI/27.).cos(); //8.*
            //     // spiral_transform.translation.y =1.*scale.y *(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
            //     // spiral_transform.translation.z =0.7*scale.z *  i as f32  *(i as f32 * PI/27.).sin();//8.*
            // }
            // else {
                // spiral_transform.translation.x =0.01* i as f32 * scale.x *  (i as f32 * PI/27.).cos(); //8.*
                // spiral_transform.translation.y =0.05 * scale.y *(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
                // spiral_transform.translation.z =0.01*  i as f32 * scale.z *  (i as f32 * PI/27.).sin();//8.*

            //     spiral_transform.translation.x =0.7*scale.x *  i as f32 *(i as f32 * PI/27.).cos(); //8.*
            //     spiral_transform.translation.y =1.*scale.y *(1.-E.powf(-1.*i as f32));//i as f32 ;//8.*
            //     spiral_transform.translation.z =0.7*scale.z *  i as f32  *(i as f32 * PI/27.).sin();//8.*
            // }

            // Spiraling Up
            spiral_pos = spiral_transform.transform_point(spiral_pos);
            // Rotate into formerly given direction
            pos = Transform::from_rotation(transform.rotation).transform_point(spiral_pos);
            // Translate to formerly given position
            pos = Transform::from_translation(transform.translation).transform_point(pos);

            // Assigning the node
            if i % extending_factor == extending_factor - 1 {
                if branches.len() > children[inner_child_index] { // To prevent len == index for /
                    let _dir = pos - last_pos;
                    
                    let mut rts = spiral_transform;

                    // Branch of from pos with last pos to pos direction
                    // rts.look_to(dir.normalize().any_orthonormal_vector(), dir);

                    // if index == *branches[branches[index].parent].children.last().unwrap() {
                    // if index == *branches[branches[index].parent].children.last().unwrap() {
                    if children[inner_child_index] == *children.last().unwrap() {
                        // rts = rts.with_translation(branches[branches[index].parent].transform.translation + Vec3 {x: 0., y: 1000., z: 0. });
                        // rts.look_to( pos.normalize().any_orthonormal_vector(), pos);//Vec3 {x: 0., y: 1., z: 0. });
                        // rts.look_to( Vec3 {x: 0., y: 1., z: 0. }, first_dir_vec.normalize());//Vec3 {x: 0., y: 1., z: 0. });
                        // rts.look_to( Vec3 {x: 0., y: 0., z: -1. },Vec3 {x: 1., y: 0., z: 0. });//Vec3 {x: 0., y: 1., z: 0. });
                        rts = rts.with_rotation(branches[branches[index].parent].transform.rotation);
                        // rts = rts.with_translation(pos + index as f32 * vec3(1., 0., 0.));
                        rts = rts.with_translation(pos);
                        rts = rts.with_scale(Vec3::splat(scale));
                        // rts.rotate_y(-PI/8.);


                        // rts = branches[branches[index].parent].transform;
                        // rts = rts.with_translation(pos);

                        // rts = rts.with_translation(Vec3 { x: 200., y: 200., z: 0. });

                        if index <100 {
                            info!("Index: {:?} Path: {:?}",index, branches[index].name);
                        }
                    }
                    else {
                        // rts.look_to((pos - transform.translation).normalize().any_orthogonal_vector(), Vec3 {x: 0., y: 1., z: 0. });
                        // rts.look_to( pos, Vec3 {x: 0., y: 1., z: 0. });//Vec3 {x: 0., y: 1., z: 0. });
                        // rts.look_to( Vec3 {x: 0., y: 0., z: -1. },Vec3 {x: 0., y: 1., z: 0. });//Vec3 {x: 0., y: 1., z: 0. });
                        // rts.look_to( dir, pos);//Vec3 {x: 0., y: 1., z: 0. });
                        // rts.look_to( Vec3 {x: 0.,y: 1.,z: 0.,}, Vec3 {x: 0., y: 1., z: -1. });
                        // rts.look_to( pos * Vec3 {x: 1.,y: 0.,z: 1.,}, Vec3 {x: 0., y: 1., z: 0. });
                        // rts.look_to(Vec3 {x: 0., y: 1., z: 0. } ,pos.normalize().any_orthogonal_vector());
                        // rts.rotate_y(PI/4.);
                        // rts.look_to( Vec3 {x: 0., y: 1., z: 0. }, first_dir_vec.normalize());
                        // rts.look_to(  Vec3 {x: 0., y: 1., z: 0. }, pos);// * Vec3 {x: 1., y: 0., z: 1. });
                        rts.look_to(  pos.normalize().any_orthogonal_vector(), pos.normalize());// * Vec3 {x: 1., y: 0., z: 1. });
                        
                        // rts.rotate_x(-PI/4.);
                        rts = rts.with_translation(pos);
                        rts = rts.with_scale(Vec3::splat(scale));
                        // rts.rotate_y(PI/16.);
                        // rts.rotate_z(PI/4.);


                    }    
                    // info!("\nIndex : {:?}\nParent: {:?}\nSiblings: {:?}",index, branches[index].parent, branches[branches[index].parent].children);

                    branches[children[inner_child_index]].transform = rts;
                    inner_child_index += 1;
                }
            }
                line_vertices.push(last_pos);
                line_vertices.push(pos);
                last_pos = pos;
        }

        for child_index in branches[index].children.clone() {
            if !(branches.len() < child_index) {
                // if child_index == *branches[index].children.last().unwrap(){
                //     info!("Name: {:?}",branches[index].name);// = "last".to_owned();
                // }
                // println!("ChildIndex: {:?} \nBranchesLen: {:?}", child_index, branches.len());
                _dive(name, child_index, branches, line_vertices);
            }
        }
    }
}


