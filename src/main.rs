//! A simple 3D scene with light shining over a cube sitting on a plane.

use std::{env, f32::consts::PI, fs, io::Read, iter::Map};

use bevy::{
    input::{keyboard::KeyCode, mouse::{MouseButtonInput, MouseMotion, MouseWheel}},
    math::{bounding::{BoundingSphere, BoundingSphereCast, BoundingVolume, IntersectsVolume, RayCast3d}, primitives::Sphere, *}, pbr::{extract_meshes, wireframe::WireframeConfig}, 
    prelude::*,
    pbr::CascadeShadowConfigBuilder,
    render::{camera::ScalingMode, mesh::{self, Indices, PrimitiveTopology}, render_resource::{AsBindGroup, ShaderRef}, view::{RenderLayers, VisibleEntities}}, transform
};


use std::str;
use walkdir::{WalkDir, DirEntry};

use std::process::Command;

mod generator;
mod database;

#[derive(Component, Debug)]
struct Cam
{
    yaw:   f32,
    pitch: f32,
    fov:   f32,
    speed: f32,
    pos: Vec3,
    rot: Quat,
}

#[derive(Component)]
struct TreeMeshMarker;
// struct linemeshmarker;

#[derive(Component)]
struct DisplayPathText;
struct Cam2D;


fn main() {
    //env::set_var("RUST_BACKTRACE", "1");

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.31, 0.51, 0.21))) // Background 2 Darkblu
        .add_systems(Startup, setup)
        .add_systems(Update, (bevy::window::close_on_esc, process_inputs_system, animate_light_direction))
        .insert_resource(AmbientLight {
            color: Color::Rgba {
                red: 0.95,
                green: 0.3,
                blue: 1.0,
                alpha:1.0,
            },
            brightness: 0.5,},
        )

        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    asset_server: Res<AssetServer>,
    window: Query<&Window>,
) {
    // Default Spawn of Scene Spawning ///////////////////////////////////////////////////////

    let mut main_tree = database::Tree::new();
    // main_tree.construct("./TestTree/Tree".to_string()); // No end "/" allowed
    // main_tree.construct("/home/nom/code/rust/storytree".to_string()); // No end "/" allowed
    // main_tree.construct("/sys".to_string()); // No end "/" allowed
    // main_tree.construct("./TestTree/Tree".to_string()); // No end "/" allowed
    main_tree.construct("/".to_string()); // No end "/" allowed
    // main_tree.construct("/sys/devices/pci0000:00".to_string()); // No end "/" allowed
    // main_tree.construct("/sys/devices/pci0000:00/0000:00:02.0/drm".to_string());
    let mut tree_name: String = "000".to_string();
    commands.spawn((PbrBundle {
        mesh: meshes.add(main_tree.grow(&mut tree_name)
    ),
        material: materials.add(
            Color::rgba(16., 0., 0., 1.0),
        ),
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
            Color::rgba(0.8, 0.4, 0.3, 1.0),
        ),
        ..Default::default()

        },
        TreeMeshMarker,
        RenderLayers::layer(0),
        )
        );

        commands.insert_resource(main_tree.clone());


    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(5000000.)),
        material: materials.add(Color::rgb(1., 1., 1.)),
        // material: materials.add(Color::rgb(0.4, 0.3, 0.4)),
        transform: Transform::from_rotation(Quat::from_rotation_x(-PI/2.)),
        ..default()
    });

    // Ui Bundle v , Probably needs Text2dBundle for precise positioning
    let font = asset_server.load("fonts/Roboto-Thin.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 18.0,
        ..default()
    };
    let text_justification = JustifyText::Center;

    // 2d camera
    commands.spawn(Camera2dBundle{camera: Camera{order:1,..default()},..Default::default()});

    let mut cnt = 0;
    for j in 0..10{
        for i in 0..20{
            cnt += 1;
            commands.spawn((
                Text2dBundle {
                    text: Text::from_section(i.to_string(), text_style.clone())
                        .with_justify(text_justification),
                    // transform: Transform::from_translation(Vec3 { x: -500. + j as f32 * 30. , y: -200. + i as f32 * 10., z: i as f32 }),
                    transform: Transform::default(),//from_translation(Vec3::splat(2220.)),
                    text_anchor: bevy::sprite::Anchor::CenterLeft,
                    ..default()
                },
                DisplayPathText,
            ));
        }
    }
    println!("Cnt: {:?}", cnt);

    // DisplayTextBundle
    // commands.spawn((
    //     // Create a TextBundle that has a Text with a single section.
    //     TextBundle::from_section(
    //         // Accepts a `String` or any type that converts into a `String`, such as `&str`
    //         "_",
    //         TextStyle {
    //             // This font is loaded and will be used instead of the default font.
    //             font: asset_server.load("fonts/Roboto-Regular.ttf"),
    //             font_size: 20.0,
    //             ..default()
    //         },
    //     ) // Set the justification of the Text
    //     .with_text_justify(JustifyText::Center)
    //     // Set the style of the TextBundle itself.
    //     .with_style(Style {
    //         position_type: PositionType::Absolute,
    //         bottom: Val::Px(5.0),
    //         left: Val::Px(5.0),
    //         ..default()
    //     }),
    //     DisplayPathText
    // )); 

    // ^^^^^^^^^^^^^^^^^
    // commands.spawn((
    //     // Create a TextBundle that has a Text with a single section.
    //     TextBundle::from_section(
    //         // Accepts a `String` or any type that converts into a `String`, such as `&str`
    //         "^",
    //         TextStyle {
    //             // This font is loaded and will be used instead of the default font.
    //             font: asset_server.load("fonts/Roboto-Regular.ttf"),
    //             font_size: 20.0,
    //             color: Color::rgba(0.7,0.6,0.6,0.5),
    //             ..default()
    //         },
    //     ) // Set the justification of the Text
    //     .with_text_justify(JustifyText::Center)
    //     // Set the style of the TextBundle itself.
    //     .with_style(Style {
    //         position_type: PositionType::Absolute,
    //         top: Val::Px(window.single().resolution.height()/2.),
    //         left: Val::Px(window.single().resolution.width()/2.),
    //         ..default()
    //     }),
    // )); 

    // Directional Light, Sunlike
    commands.spawn((DirectionalLightBundle {
        directional_light: DirectionalLight {
                        color: Color::Rgba {
                        red: 0.7,
                        green: 0.4,
                        blue: 0.1,
                        alpha:0.1,
            },
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-PI/2.)),

        // transform: Transform::from_xyz(0.,-200.,0.)*Transform::from_rotation(Quat::from_rotation_x(PI/2.)),
        ..default()
    },
    // RenderLayers::all(),)
    ));




    let x = 1.0;
    let y = 1.0;
    let z = 2.0;
    let w = 1.0;

    // Working classic cam
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0., 10., 40.0).looking_at(Vec3::ZERO, Vec3::Y),
        projection: PerspectiveProjection {
            // fov: (90.0 / 360.0) * (std::f32::consts::PI * 2.0),
            // aspect_ratio: 1.0,
            ..default()
        }.into(),
        ..default()
    },
        Cam {yaw: 0., pitch: 0., fov: 1.0, speed:0.4, pos: Vec3::ZERO, rot: Quat::from_xyzw(0.0, 0.0, 0.0, 1.0)},
        // RenderLayers::all(),
    ));

    // new 3D orthographic camera
    // commands.spawn((Camera3dBundle {
    //     projection: OrthographicProjection {
    //         scale: 0.1,
    //         //scaling_mode: ScalingMode::FixedVertical(15.0),
    //         ..default()
    //     }.into(),
    //     ..default()
    // },
    //         Cam {yaw: 0., pitch: 0., fov: 1.0, speed:0.2, pos: Vec3::ZERO, rot: Quat::from_xyzw(0.0, 0.0, 0.0, 1.0)},
    // ));
    
}

fn process_inputs_system(
    // Cam_Control
    keys: Res<ButtonInput<KeyCode>>,
    mut q_transform: Query<&mut Transform, With<Cam>>,
    mut q_cam: Query<&mut Cam>,  
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut q_pp: Query<&mut Projection, With<Cam>>,

    // Update_Scale
    mut tree: Query<(&mut Transform, &TreeMeshMarker), Without<Cam>>,
    mut tree_data: Option<ResMut<database::Tree>>,

    // Node_Picking
    buttons: Res<ButtonInput<MouseButton>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Cam>>,
    q_window: Query<&Window>,
    // mut display_text_query: Query<&mut Text, With<DisplayPathText>>,

    mut q_screen_text_transform: Query<(&mut Text, &mut Transform), (With<Text>, With<DisplayPathText>, Without<Cam>, Without<TreeMeshMarker>)>,

    //VisualDebugging
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    // Single Instance to avoid iterating queue
    let mut cam = q_cam.single_mut();

    // Update Cam Yaw & Pitch  // adjust for very small fovs
    for event in mouse_motion_events.read() {
            cam.yaw   += -event.delta.x * cam.fov; 
            cam.pitch += -event.delta.y * cam.fov;     
    }

    // Update transform from keyboardinput and Yaw&Pitch
    for mut transform in q_transform.iter_mut() {

        let mut temp_transform: Transform = Transform{ translation: transform.translation, rotation: transform.rotation, scale: transform.scale,};

        // Calculate rotation
        temp_transform.rotation = Quat::from_rotation_y(cam.yaw * 0.005)* Quat::from_rotation_x(cam.pitch * 0.005)  ;  

        // Tastatursteuerung, deltatranslation
        let translation_delta = {
            let mut delta = Vec3::ZERO;
            if keys.pressed(KeyCode::KeyW) {
                delta.z -= cam.speed;
            }
            if keys.pressed(KeyCode::KeyS) {
                delta.z += cam.speed;
            }
            if keys.pressed(KeyCode::KeyA) {
                delta.x -= cam.speed;
            }
            if keys.pressed(KeyCode::KeyD) {
                delta.x += cam.speed;
            }
            if keys.pressed(KeyCode::KeyQ) {
                delta.y -= cam.speed;
            }
            if keys.pressed(KeyCode::KeyE) {
                delta.y += cam.speed;
            }

            // Manual Zoom 1. -> 0.1 -> 0.01 -> 0.001 -> ..
            if keys.pressed(KeyCode::Space) { // Zoom In
                let mut i = 1;
                while (1. - cam.fov * 10.0_f32.powf(i as f32)) >= 0. {
                    i += 1;
                }
                cam.fov -= 0.1_f32.powf(i as f32);

            }
            if keys.pressed(KeyCode::AltLeft) { // Zoom Out
                let mut i = 0;

                if cam.fov < 1. {
                    while (1. - cam.fov * 10.0_f32.powf(i as f32)) >= 0. {
                        i += 1;
                        println!("{:?} :: {:?}",cam.fov,  i);
                    }
                    cam.fov += 0.1_f32.powf(i as f32);
                }   
                else {
                    i = 1;
                    while (1. - cam.fov / 10.0_f32.powf(i as f32)) <= 0. {
                        i += 1;
                        println!("{:?} :: {:?}",cam.fov,  i);
                    }
                    cam.fov += 0.1_f32.powf(i as f32);
                }    

                if cam.fov > 3.0 {
                    cam.fov = 3.0
                }
                else {
                    cam.fov += 0.1_f32.powf(i as f32);
                }
            }

            // Adjust Speed
            if keys.pressed(KeyCode::ShiftLeft) {
                cam.speed += 1.07;
            }
            if keys.pressed(KeyCode::ControlLeft) {
                cam.speed -= 1.07;

                // treebuilder::print_hello();
            }
            delta
        };

        // Update actual cam transformation
        temp_transform.translation += temp_transform.rotation * translation_delta;
        *transform = temp_transform;

        // Update cam position where we are
        cam.pos = transform.translation;
        cam.rot = transform.rotation;

        // Update actual fov / perspective / to zoom to an extend
        for mut pp in q_pp.iter_mut() {
            
            // Perspective Projection Update
            *pp = PerspectiveProjection {
                fov: cam.fov,
                aspect_ratio: 1.0,
                ..default()
            }.into()

            // Orthographic Projection Update
            // *pp = OrthographicProjection {
            //     scale: cam.fov,
            //     ..Default::default()
            // }.into()
        }
    }


    // Update_Scale 
    if keys.pressed(KeyCode::Digit1) {
        for (mut transform, cube) in &mut tree {
            transform.scale *= Vec3{x: 0.9,y:0.9,z: 0.9};
            // transform.translation.y += 2. * transform.scale.y;
        }
    }

    if keys.just_released(KeyCode::Digit1) {
        for (transform, cube) in &mut tree {
        if let Some(tree_data) = &mut tree_data {
            // println!("Radius: {:?}", tree_data.bounds[2].sphere.radius);
            for i in 0..tree_data.bounds.len(){
                tree_data.bounds[i].center = tree_data.branches[i].transform.transform_point(Vec3::splat(0.));
                tree_data.bounds[i].center *= transform.scale;
                tree_data.bounds[i].sphere.radius = transform.scale.y + transform.scale.y*0.2;
            }
            }
        }
    }
    
    if keys.pressed(KeyCode::Digit2) {
        for (mut transform, cube) in &mut tree {
            transform.scale *= Vec3{x: 1.1,y:1.1,z: 1.1};
            // transform.translation.y -= 2. * transform.scale.y;
        }
    }

    if keys.just_released(KeyCode::Digit2) {
        for (transform, cube) in &mut tree {
        if let Some(tree_data) = &mut tree_data {
            for i in 0..tree_data.bounds.len(){
                tree_data.bounds[i].center = tree_data.branches[i].transform.transform_point(Vec3::splat(0.));
                tree_data.bounds[i].center *= transform.scale;
                tree_data.bounds[i].sphere.radius = transform.scale.y + transform.scale.y*0.2;
                }
                }
            }
        }


    // Reset to pos
    if keys.just_released(KeyCode::Tab) {
            let mut cam_transform = q_transform.single_mut();
        
            let pos = vec3(0., 5000., 10000.);
            cam_transform.translation = pos;
    
            // Teleport camera to clicked place
            info!("Reset performed. Cameratransform: {:?}", cam_transform);       
    }

    
    // Node_Picking
    let window = q_window.single();
    let (camera, mut camera_transform) = q_camera.single();
    let mut q_screen_text = q_screen_text_transform.iter_mut();

    if buttons.just_released(MouseButton::Left) {
        // let window = q_window.single();
        // let (camera, camera_transform) = q_camera.single();
        
         //   for (transform, marker) in &mut tree {

                if let Some(cam_ray) = window.cursor_position().and_then(|cursor| camera.viewport_to_world(camera_transform, cursor)){

                    if let Some(tree_data) = &mut tree_data {

                        let ray_cast = RayCast3d::from_ray(cam_ray, 10000.);

                        for (i, branch, ) in tree_data.bounds.clone().into_iter().enumerate(){
                            let cast_result = ray_cast.intersects(&branch);//BoundingSphereCast::from_ray(branch, world_position, 10000.);
                            if cast_result == true {
                                
                                // if let Some((mut text,mut text_transform)) = q_screen_text.next(){
                                //         text_transform.translation.x = 0.;
                                //         text_transform.translation.y = 0.;
                                //         text.sections[0].value = tree_data.branches[i].name.clone();
                                // }

                                //Terminalmagic

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
                                
                                info!("TreeBranchName: {:?}",tree_data.branches[i].name);

                                let mut cnt = 0.;
                                for entry in WalkDir::new(tree_data.branches[i].name.clone().to_string()).max_depth(1).into_iter().filter_map(|e| e.ok()) {
                                    println!(": {:?} :", entry.path());
                                    if entry.file_type().is_file() || entry.file_type().is_symlink() {
                                        if let Some((mut text,mut text_transform)) = q_screen_text.next(){
                                            text_transform.translation.x = -200.;
                                            text_transform.translation.y = 200. - 10.* cnt;
                                            text.sections[0].value = entry.file_name().to_str().unwrap().to_string();
                                            cnt += 2.;
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
                    }
                }
           // }
    }

    // Cam teleport to right click, somehow kills later klick detection TODO
    if buttons.just_released(MouseButton::Right) {

        if let Some(cam_ray) = window.cursor_position().and_then(|cursor| camera.viewport_to_world(camera_transform, cursor)){

            if let Some(tree_data) = &mut tree_data {

                let ray_cast = RayCast3d::from_ray(cam_ray, 10000.);

                for (i, branch, ) in tree_data.bounds.clone().into_iter().enumerate(){
                    let cast_result = ray_cast.intersects(&branch);//BoundingSphereCast::from_ray(branch, world_position, 10000.);
                    if cast_result == true {

                        let mut cam_transform = q_transform.single_mut();
                        
                        cam_transform.translation = branch.center;//Vec3::splat(1000.);//0.9 * (cam_transform.translation - branch.center);//branch.center - cam.pos;
                        cam.pos = cam_transform.translation;

                        // Teleport camera to clicked place
                        info!("Teleporting camera, Cameratransform: {:?}", cam.pos);
                    }
                }
            }
        }
    }



    // #TODO:
    // Updates node information in cameraview // Depending on Bounding sphere, needs second check if bounding finds are really in view
    if let Some(tree_data) = &mut tree_data {
        // Places the bounding sphere a bit in front of the camera, bounding box at viewplane may be more usefull
        // let bound_sphere = BoundingSphere::new(camera_transform.forward().mul_add(Vec3 { x: 0., y: 0., z: 6000. }, camera_transform.translation()), 10000.0);
        let bound_sphere = BoundingSphere::new(camera_transform.transform_point(camera_transform.to_scale_rotation_translation().1.mul_vec3(Vec3 { x: 0., y: 0., z: 6000. })), 10000.0);
        
        // let mut q_screen_text = q_screen_text_transform.iter_mut();
        for (i, branch, ) in tree_data.bounds.clone().into_iter().enumerate(){
            let cast_result = bound_sphere.intersects(&branch);//BoundingSphereCast::from_ray(branch, world_position, 10000.);
            if cast_result == true {

                // println!("#{} CastResult: {:?} Path: {:?} \n BranchInfo: {:?} \n", i, cast_result, tree_data.branches[i].name, tree_data.branches[i]);

                if let Some((mut text,mut text_transform)) = q_screen_text.next(){
                    if let Some(screen_position) = camera.world_to_ndc(camera_transform, tree_data.bounds[i].center){

                        text_transform.translation.x = screen_position.x * window.width() *0.5;// - window.width()/2.;
                        text_transform.translation.y = screen_position.y * window.height()*0.5;// - window.height()/2.;
                        text_transform.translation.z = screen_position.z;

                        // #TODO:
                        // Following should be Switchable

                        // Whole path
                        // text.sections[0].value = tree_data.branches[i].name.clone();// + " " + &screen_position.x.to_string() + " " + &screen_position.y.to_string();
                        
                        // Only Current folder
                        text.sections[0].value = tree_data.branches[i].name.clone().rsplit_once("/").unwrap().1.to_string();
                    }
                }
            }
        }
        while let Some((_text,mut text_transform))  = q_screen_text.next(){
            text_transform.translation.x = -window.width();
            text_transform.translation.y = window.height();
        }               
    }
}

// --- // --- // Utils \\ --- \\ --- \\

fn count_entities(all_entities: Query<()>) {
    dbg!(all_entities.iter().count());
}

fn print_typename<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
    mut q_cam: Query<&mut Cam>, 
) {
    // println!("Cam: {:?}", q_cam.single_mut().pos);
    for mut transform in &mut query {
        
        // *transform = Transform::from_rotation(q_cam.single_mut().rot) * Transform::from_xyz(0.0, q_cam.single_mut().pos.y+0.0, 0.0);
        // *transform = Transform::from_rotation(q_cam.single_mut().rot) * Transform::from_xyz(q_cam.single_mut().pos.x, q_cam.single_mut().pos.y+0.0, q_cam.single_mut().pos.z);

        // transform.rotate_y(time.delta_seconds() * 0.5);
    }
}