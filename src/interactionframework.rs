use core::f32;

use bevy::{
    prelude::*,
    input::{keyboard::KeyCode, mouse::MouseMotion},
    math::{bounding::{BoundingSphere, IntersectsVolume, RayCast3d}, primitives::Sphere, *}, //pbr::{extract_meshes, wireframe::WireframeConfig, CascadeShadowConfigBuilder}, 
    render::view::RenderLayers,
    utils::HashSet
};

 
#[derive(Component, Debug)]
pub struct Cam
{
    pub yaw:   f32,
    pub pitch: f32,
    pub fov:   f32,
    pub speed: f32,
    pub pos:   Vec3,
    pub rot:   Quat,
}

use walkdir::WalkDir;
use generator::GenerationType;

use crate::database;
use crate::generator;

#[derive(Component)]
pub struct DisplayPathText;

#[derive(Component)]
pub struct TreeMeshMarker;

pub fn process_inputs_system(
    // Cam_Control
    keys: Res<ButtonInput<KeyCode>>,
    mut q_cam_transform: Query<&mut Transform, With<Cam>>,
    mut q_cam: Query<&mut Cam>,  

    mut mouse_motion_events: EventReader<MouseMotion>,
    mut q_perspektiveprojection: Query<&mut Projection, With<Cam>>,

    // Update_Scale
    mut tree: Query<&mut Transform, (With<TreeMeshMarker>, Without<Cam>)>,
    mut tree_data: Option<ResMut<database::Tree>>,

    // Node_Picking
    buttons:  Res<ButtonInput<MouseButton>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Cam>>,
    q_window: Query<&Window>,

    mut q_screen_text_transform: Query<(&mut Text, &mut Transform), 
        (With<Text>, With<DisplayPathText>, Without<Cam>, Without<TreeMeshMarker>)>,

    //VisualDebugging
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    mut input_path: Local<String>,
    mut enter_cnt: Local<f32>
) {
    // Single Instance to avoid iterating queue
    let mut cam = q_cam.single_mut();

    // Update Cam Yaw & Pitch  // adjusted for very small fovs
    for event in mouse_motion_events.read() {
            cam.yaw   += -event.delta.x * cam.fov; 
            cam.pitch += -event.delta.y * cam.fov; 
    }

    // Update transform from keyboardinput and Yaw&Pitch
    for mut transform in q_cam_transform.iter_mut() {

        let mut temp_transform: Transform = Transform{ translation: transform.translation, rotation: transform.rotation, scale: transform.scale,};

        // Calculate rotation
        temp_transform.rotation = Quat::from_rotation_y(cam.yaw   * 0.005)
                                * Quat::from_rotation_x(cam.pitch * 0.005); 

        // Adjust Zoom and Speed
        if keys.pressed(KeyCode::Space      ) { cam.fov   /= 1.25;} // Zoom In
        if keys.pressed(KeyCode::AltLeft    ) { cam.fov   *= 1.25;} // Zoom Out
        if keys.pressed(KeyCode::ShiftLeft  ) { cam.speed *= 1.25;} // Speed Up
        if keys.pressed(KeyCode::ControlLeft) { cam.speed /= 1.25;} // Speed Down

        // Tastatursteuerung, deltatranslation
        let translation_delta = {
        let mut delta = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {delta.z -= cam.speed;}
        if keys.pressed(KeyCode::KeyS) {delta.z += cam.speed;}
        if keys.pressed(KeyCode::KeyA) {delta.x -= cam.speed *cam.fov;}
        if keys.pressed(KeyCode::KeyD) {delta.x += cam.speed *cam.fov;}
        if keys.pressed(KeyCode::KeyQ) {delta.y -= cam.speed *cam.fov;}
        if keys.pressed(KeyCode::KeyE) {delta.y += cam.speed *cam.fov;}
        delta};

        // Update actual cam transformation
        temp_transform.translation += temp_transform.rotation * translation_delta;
        *transform = temp_transform;

        // Update cam position where we are
        cam.pos = transform.translation;
        cam.rot = transform.rotation;

        // Update actual fov / perspective / to zoom 
        let mut pp = q_perspektiveprojection.single_mut();
        *pp = PerspectiveProjection {fov: cam.fov, aspect_ratio: 1.0, ..default()}.into();

    }

    //     // Update actual fov / perspective / to zoom to an extend
    //     for mut pp in q_perspektiveprojection.iter_mut() {
    //         *pp = PerspectiveProjection {
    //             fov: cam.fov,
    //             aspect_ratio: 1.0,
    //             ..default()
    //         }.into()

    //         // Orthographic Projection Update
    //         // *pp = OrthographicProjection {
    //         //     scale: cam.fov,
    //         //     ..Default::default()
    //         // }.into()
    //     }
    // }

    // Update_Scale 
    if keys.pressed(KeyCode::Digit1) {
        for mut transform in &mut tree {
            transform.scale *= Vec3{x: 0.9,y:0.9,z: 0.9};}
    }

    if keys.just_released(KeyCode::Digit1) {
        for transform in &mut tree {
            if let Some(tree_data) = &mut tree_data {
                for i in 0..tree_data.bounds.len(){
                    tree_data.bounds[i].center = tree_data.branches[i].transform.translation;
                    tree_data.bounds[i].center *= transform.scale;
                    tree_data.bounds[i].sphere.radius = tree_data.branches[i].transform.scale.y * transform.scale.y * 7.;
                }
            }
        }
    }
    
    if keys.pressed(KeyCode::Digit2) {
        for mut transform in &mut tree {
            transform.scale *= Vec3{x: 1.1,y:1.1,z: 1.1};
        }
    }

    if keys.just_released(KeyCode::Digit2) {
        for transform in &mut tree {
            if let Some(tree_data) = &mut tree_data {
                for i in 0..tree_data.bounds.len(){
                    tree_data.bounds[i].center = tree_data.branches[i].transform.translation;
                    tree_data.bounds[i].center *= transform.scale;
                    tree_data.bounds[i].sphere.radius = tree_data.branches[i].transform.scale.y * transform.scale.y * 7.;
                }
            }
        }
    }

    // Reset to pos
    if keys.just_released(KeyCode::Tab) {
            let mut cam_transform = q_cam_transform.single_mut();
        
            let pos = vec3(0., 7000., 15000.);
            cam_transform.translation = pos;
    
            // Teleport camera to clicked place
            info!("Reset performed. Cameratransform: {:?}", cam_transform);       
    }

    // Enter Tree Generation
    if keys.just_released(KeyCode::Enter){//input_path.is_empty() {
        // if *enter_cnt == 1. {
        //     spawn_tree("/sys".to_string(), Vec3 { x: 0., y: 0., z: 0. },  &mut commands, &mut meshes,&mut materials);
        //     *enter_cnt = 1.;
        // }
        // else if *enter_cnt == 0. {
        //     spawn_generator_tree("/sys".to_string(), Vec3 { x: 0., y: 0., z: 0. }, &mut commands, &mut meshes,&mut materials, true, true, 30);
        //     *enter_cnt += 1.;
        // }
        // else {
        //     *enter_cnt += 1.;
        //     info!("Path: {:?}", input_path.clone());
        //         for i in -15..15 {
        //             spawn_tree(input_path.clone().to_string(), Vec3 { x: 10000. * *enter_cnt, y: 0., z:  50000. * i as f32 }
        //             , &mut commands, &mut meshes,&mut materials);
        //         }
        // }
        *enter_cnt += 1.;
    }

    if keys.just_released(KeyCode::Backspace){
        
        let center = calc_variance(&tree_data.as_ref().unwrap().bounds);
        let scale = 0.1;
        println!("{:?} {:?}",calc_fractal_dimension(&tree_data.as_ref().unwrap().bounds, 1.), 1.);
        println!("{:?} {:?}",calc_fractal_dimension(&tree_data.as_ref().unwrap().bounds, 2.), 2.);
        println!("{:?} {:?}",calc_fractal_dimension(&tree_data.as_ref().unwrap().bounds, 3.), 3.);
        println!("{:?} {:?}",calc_fractal_dimension(&tree_data.as_ref().unwrap().bounds, 4.), 4.);
        println!("{:?} {:?}",calc_fractal_dimension(&tree_data.as_ref().unwrap().bounds, 5.), 5.);

        commands.spawn(PbrBundle {
            mesh: meshes.add(Sphere::new(100.)),
            material: materials.add(Color::rgb(0., 1., 0.)),
            transform: Transform::from_xyz(center.x, center.y, center.z),//Transform::from_rotation(Quat::from_rotation_x(-PI/2.)),
            ..default()
        });
    }

    // Node_Picking // Terminal possibiltys
    let window = q_window.single();
    let (camera, camera_transform) = q_camera.single();
    let mut q_screen_text = q_screen_text_transform.iter_mut();

    if buttons.just_released(MouseButton::Left) {
        if let Some(cam_ray) = window.cursor_position().and_then(|cursor| camera.viewport_to_world(camera_transform, cursor)){
            if let Some(tree_data) = &mut tree_data {
                let ray_cast = RayCast3d::from_ray(cam_ray, f32::MAX);
                for (i, branch, ) in tree_data.bounds.clone().into_iter().enumerate(){
                    let cast_result = ray_cast.intersects(&branch);
                    if cast_result == true {
                        let path_name = tree_data.branches[i].name.clone();
                        *input_path = path_name.clone();

                        let mut cnt_x = 50.;
                        let mut cnt_y = 50.;
                        for entry in WalkDir::new(tree_data.branches[i].name.clone().to_string()).max_depth(1).into_iter().filter_map(|e| e.ok()) {
                            // Lists all files and symlinks in an encountered node in the top left corner as 2D text
                            println!(": {:?} :", entry.path());
                            if entry.file_type().is_file() || entry.file_type().is_symlink() {
                                if let Some((mut text,mut text_transform)) = q_screen_text.next(){ 
                                    text_transform.translation.x = -window.width()*0.5 + cnt_x;
                                    text_transform.translation.y = window.height()*0.5 - cnt_y;
                                    if (cnt_y == 50.){
                                        text.sections[0].value = path_name.clone();//stripped_path;//entry.path().to_str().unwrap().rsplit_once("/").unwrap();//.to_string(); // Display path on top
                                        cnt_y += 40.;
                                    }
                                    else {
                                        text.sections[0].value = entry.file_name().to_str().unwrap().to_string();
                                        cnt_y += 20.;

                                        if cnt_y > 400. {
                                            cnt_x += 50.;
                                            cnt_y = 90.;
                                        }
                                    }
                                }
                            }
                            else {
                                info!("No files here. Path: {:?}", entry.path());
                            }
                        }
                        // Click on node to translate all points around the center point by substracting clicked point from every point?
                        // To then be able to scale from it 
                    }
                }
                println!("LeftMouseButton release bounds loop finished: Iterating files \n");
                // https://bevy-cheatbook.github.io/programming/commands.html for despawning and spawning file representatives
            }
        }
    }

    //Terminalmagic - inside found cast result ^

    // Cat Example
    // let output = Command::new("/bin/cat")
    //                     .arg("file.txt")
    //                     .output()
    //                     .expect("failed to execute process");
    // println!("status: {}", output.status);
    // println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    // println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    // assert!(output.status.success());

    // Create Folder with a click
    // let output = Command::new("mkdir")
    // .arg(tree_data.branches[i].name.clone().to_string() + "/OoOoO")
    // .spawn()
    // .expect("Mkdir command failed to start");

    // Ls
    // let mut output = Default::default();
    // let ls = Command::new("ls")
    // .arg(tree_data.branches[i].name.clone().to_string())
    // .spawn()
    // .expect("ls command failed to start")
    // .stdout.take().unwrap().read_to_string(&mut output);

    // List files with click // Needs own word list or 3d spawn in world, right now only blinking shortly but it works
    

    // Cam teleport to right click
    if buttons.just_released(MouseButton::Right) {
        if let Some(cam_ray) = window.cursor_position().and_then(|cursor| camera.viewport_to_world(camera_transform, cursor)){
            if let Some(tree_data) = &mut tree_data {
                let ray_cast = RayCast3d::from_ray(cam_ray, f32::MAX);
                for (_i, branch, ) in tree_data.bounds.clone().into_iter().enumerate(){
                    let cast_result = ray_cast.intersects(&branch);//BoundingSphereCast::from_ray(branch, world_position, 10000.);
                    if cast_result == true {
                        let mut cam_transform = q_cam_transform.single_mut();
                        
                        cam_transform.translation = branch.center;//Vec3::splat(1000.);//0.9 * (cam_transform.translation - branch.center);//branch.center - cam.pos;
                        cam.pos = cam_transform.translation;

                        // Teleport camera to clicked place
                        info!("Teleporting camera, Cameratransform: {:?}", cam.pos); 
                    }
                }
            }
        }
    }

    // Update Node texts
    // #TODO:
    // Updates node information in cameraview // Depending on Bounding sphere
   // Places the bounding sphere a bit in front of the camera, bounding box at viewplane may be more usefull // Should be adjustable from some panel
    // let cam_bounding_sphere = BoundingSphere::new(camera_transform.forward().mul_add(Vec3 { x: 0., y: 0., z: 6000. }, camera_transform.translation()), 10000.0);
    
    // Rangescan, continiously casts a sphere intersection test around the user to display near nodes
    if let Some(tree_data) = &tree_data {
 
        let cam_bounding_sphere = BoundingSphere::new(
            camera_transform.transform_point(camera_transform.to_scale_rotation_translation().1
            .mul_vec3(Vec3 { x: 1., y:1., z: 1. })), 5000. );//* (1.+*enter_cnt)); 
        
        // Storage vector for Name, Distance and Screenposition of detected nodes
        let mut detected_nodes: Vec<(String, i32, Vec3)> = vec![];
        
        // Iterates and enmuerates all branches and checks if the created bounding sphere intersects with any node sphere
        for (i, node_bounding_sphere, ) in tree_data.bounds.clone().into_iter().enumerate(){
            // If a match is found, translates world coordinates to screen coordinates
            if cam_bounding_sphere.intersects(&node_bounding_sphere) == true { 
                    // Calculates the dot product to assure to only display node names in front of the camera
                    let distance_vec = tree_data.bounds[i].center - camera_transform.translation();
                    let dot_product = distance_vec.normalize().dot(camera_transform.forward());
                    if  dot_product > 0. { 
                        if let Some(screen_position) = camera.world_to_ndc(camera_transform, tree_data.bounds[i].center){
                            let mut adjusted_screen_pos: Vec3 = Vec3::default();
                            adjusted_screen_pos.x = screen_position.x * window.width() *0.5;
                            adjusted_screen_pos.y = screen_position.y * window.height()*0.5;
                            adjusted_screen_pos.z = screen_position.z;
                            let distance = camera_transform.translation().distance(tree_data.bounds[i].center) as i32;        
                        if *enter_cnt as i32 % 2 == 0 {
                            detected_nodes.push( ( // Displays split off nodename from path
                                "/".to_owned() + tree_data.branches[i].name.clone().rsplit_once("/").unwrap().1,
                                distance, adjusted_screen_pos))
                        }
                        else {
                            detected_nodes.push( ( // Displays whole path
                                tree_data.branches[i].name.clone(),distance,adjusted_screen_pos)) }}}}}


        //                 }
        //             }
        //         }
        //     }
        // }

        if detected_nodes.len() > 0 {
            // Sorts by distance to assure nearest nodes are displayed 
            detected_nodes.sort_by_key(|k| k.1); 
            let mut node_iter = detected_nodes.clone().into_iter();

            // Updates text and positional values of the screentext buffer to display the node names
            while let Some((mut text,mut text_transform)) = q_screen_text.next(){
                if let Some(screentext_package) = node_iter.next(){
                    text.sections[0].value = screentext_package.0.clone();
                    text_transform.translation = screentext_package.2;
                }
                else {
                    text_transform.translation.x = -window.width();
                    text_transform.translation.y = window.height();
                }
            }  
        }

        // Pushes textartefacts out of view 
        while let Some((_text,mut text_transform)) = q_screen_text.next(){
            text_transform.translation.x = -window.width();
            text_transform.translation.y = window.height();
        }               
    }
}

    // // Updates text and positional values of the screentext buffer to display the node names
    // while let Some((mut text,mut text_transform)) = q_screen_text.next(){
    //     if let Some(screentext_package) = node_iter.next(){
    //         text.sections[0].value = screentext_package.0.clone();
    //         text_transform.translation = screentext_package.2;
    //     }
    //     else {
    //         text_transform.translation.x = -window.width();
    //         text_transform.translation.y = window.height();
    //     }
    // }  

    // Dot Product calculates the cos(angle) between to vecs
    // -1 : Pointing in opposite directions
    //  0 : Perpendicular
    //  1 : Exactly same direction

                //         // #TODO:
                //         // Following should be Switchable

                //         // Whole path
                //         // text.sections[0].value = tree_data.branches[i].name.clone();// + " " + &screen_position.x.to_string() + " " + &screen_position.y.to_string();
                        
                //         // Only Current folder
                //         text.s

            //     while let Some((mut text,mut text_transform)) = q_screen_text.next(){
            //         if let Some(package) = node_iter.next(){//nth(cnt) {
            //             text.sections[0].value = package.0.clone();
            //             text_transform.translation = package.2;
            //         }
            //         else {
            //             text_transform.translation.x = -window.width();
            //             text_transform.translation.y = window.height();
            //         }
            //     }  
            // }

pub fn spawn_tree (
    path: String,
    pos: Vec3,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,

    // asset_server: Res<AssetServer>,
    // window: Query<&Window>,

) {
    let mut main_tree = database::Tree::new();

    // main_tree.construct(path.clone()); 
    // main_tree.construct("./TestTree/Tree".to_string()); // No end "/" allowed
    
    commands.spawn((PbrBundle { // Linemesh
        mesh: meshes.add(main_tree.construct(path.clone())//grow(&mut tree_name, param_set)
    ),
        material: materials.add(
            Color::rgba_linear(55., 12., 0.0, 0.9)//rgb(1., 1., 1.),
        ),
        transform: Transform::from_translation(pos),
        ..Default::default()
        },
        TreeMeshMarker,
        RenderLayers::layer(0),
        )
        );

    // Dodecas
    commands.spawn((PbrBundle {
        mesh: meshes.add(main_tree.mesh_nodes()
    ),
        material: materials.add(
            Color::rgba_linear(1., 17., 1., 0.5)//rgb(1., 1., 1.),
        ),
        transform: Transform::from_translation(pos),
        ..Default::default()
        },
        TreeMeshMarker,  // Enable this to also scale nodes
        RenderLayers::layer(0),
        )
        );

        commands.insert_resource(main_tree.clone());
    }

    pub fn spawn_generator_tree (    
            path: String,
            pos: Vec3,
            commands: &mut Commands,
            meshes: &mut ResMut<Assets<Mesh>>,
            materials: &mut ResMut<Assets<StandardMaterial>>,
            textflag: bool,
            dodecaflag: bool, 
            depth: usize
        ) {

        let (text_mesh, space_mesh, line_mesh) = generator::walk_path_to_mesh(path.as_ref(), GenerationType::Branch, depth, textflag, dodecaflag);

        let mem_text = text_mesh.count_vertices()   ;//* 3 * core::mem::size_of::<f32>();
        let mem_space = space_mesh.count_vertices() ;//* 3 * core::mem::size_of::<f32>();
        let mem_line = line_mesh.count_vertices()   ;//* 3 * core::mem::size_of::<f32>();

        info!(">Count, total nodes, line, space, text,Size total:{:?} {:?} {:?} {:?} {:?} mega bytes", mem_space/20,  mem_line, mem_space, mem_text, ((mem_text+mem_space+mem_line) as f32 * 3. * core::mem::size_of::<f32>() as f32)/1024.);

        commands.spawn((PbrBundle {
            mesh: meshes.add(line_mesh),
            material: materials.add(
                Color::rgba(0.8, 0.4, 0.3, 1.0),
            ),
            transform: Transform::from_translation(pos),
            ..Default::default()
            },
            TreeMeshMarker,  // Enable this to also scale nodes 
            RenderLayers::layer(0),
            )
            );

        commands.spawn((PbrBundle {
            mesh: meshes.add(space_mesh),
            material: materials.add(
                Color::rgba(0.8, 0.4, 0.3, 1.0),
            ),
            // transform: Transform::from_translationpos),
            transform: Transform::from_translation(pos),
            ..Default::default()
            },
            TreeMeshMarker,  // Enable this to also scale nodes
            RenderLayers::layer(0),
            )
            );

        commands.spawn((PbrBundle {
            mesh: meshes.add(text_mesh),
            material: materials.add(
                Color::rgba(0.8, 0.4, 0.3, 1.0),
            ),
            // transform: Transform::from_translationpos),
            transform: Transform::from_translation(pos),
            ..Default::default()
            },
            TreeMeshMarker,  // Enable this to also scale nodes
            RenderLayers::layer(0),
            )
            );
    }

// --- // --- // Utils \\ --- \\ --- \\

fn calc_variance(bounds:& Vec<BoundingSphere>) -> Vec3 {
    let mut cnt = 0;
    let mut focal_point = Vec3::default();
    for sphere in bounds {
        focal_point += sphere.center;
        // info!("{:?} Spehre: {:?} ",cnt, sphere.center);
        cnt += 1;
    }
    let div: f32 = cnt as f32;
    focal_point.x = focal_point.x / div;
    focal_point.y = focal_point.y / div;
    focal_point.z = focal_point.z / div;

    let mut variance = Vec3::default();
    for sphere in bounds {

        variance.x += (sphere.center.x - focal_point.x).powf(2.);
        variance.y += (sphere.center.y - focal_point.y).powf(2.);
        variance.z += (sphere.center.z - focal_point.z).powf(2.);
    }

    variance.x = variance.x / div;
    variance.y = variance.y / div;
    variance.z = variance.z / div;

    variance.x = variance.x.sqrt();
    variance.y = variance.y.sqrt();
    variance.z = variance.z.sqrt();

    info!("Variance: {:?} FocalPoint: {:?}", variance, focal_point);
    variance
}

fn calc_fractal_dimension(spheres: & Vec<BoundingSphere>, box_size: f64) -> usize {
    let mut unique_boxes: HashSet<(i64, i64, i64)> = HashSet::new();
    for sphere in spheres {
        unique_boxes.insert(to_box(sphere.center,box_size));
    }
    unique_boxes.len()
}

fn to_box(pos: Vec3, box_size: f64) -> (i64, i64, i64) {
    (
        (pos.x as f64 / box_size).floor() as i64,
        (pos.y as f64 / box_size).floor() as i64,
        (pos.z as f64 / box_size).floor() as i64,
    )
}


    // Old Infinty zoom
    // Manual Zoom 1. -> 0.1 -> 0.01 -> 0.001 -> ..
    // if keys.pressed(KeyCode::Space) { // Zoom In
    //     let mut i = 1;
    //     while (1. - cam.fov * 10.0_f32.powf(i as f32)) >= 0. {
    //         i += 1;
    //     }
    //     cam.fov -= 0.1_f32.powf(i as f32);
    // }
    // if keys.pressed(KeyCode::AltLeft) { // Zoom Out
    //     let mut i = 0;

    //     if cam.fov < 1. {
    //         while (1. - cam.fov * 10.0_f32.powf(i as f32)) >= 0. {
    //             i += 1;
    //         }
    //         cam.fov += 0.1_f32.powf(i as f32);
    //     }   
    //     else {
    //         i = 1;
    //         while (1. - cam.fov / 10.0_f32.powf(i as f32)) <= 0. {
    //             i += 1;
    //         }
    //         cam.fov += 0.1_f32.powf(i as f32);
    //     }    

    //     if cam.fov > 3.0 {
    //         cam.fov = 3.0
    //     }
    //     else {
    //         cam.fov += 0.1_f32.powf(i as f32);
    //     }
    // }
