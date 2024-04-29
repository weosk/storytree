//! A simple 3D scene with light shining over a cube sitting on a plane.

use std::{env, f32::consts::PI, fs, iter::Map};

use bevy::{
    input::{keyboard::KeyCode, mouse::{MouseButtonInput, MouseMotion, MouseWheel}},
    math::{bounding::{BoundingSphere, BoundingSphereCast, BoundingVolume, IntersectsVolume, RayCast3d}, primitives::Sphere, *}, pbr::{extract_meshes, wireframe::WireframeConfig}, 
    prelude::*,
    render::{camera::ScalingMode, mesh::{self, Indices, PrimitiveTopology}, render_resource::{AsBindGroup, ShaderRef}, view::RenderLayers}
};

use walkdir::WalkDir;

// mod textgenerator;

// mod treedata;
// use treedata::Treedata;

mod generator;
mod database;
// use generator::generate_space_mesh;

// mod treebuilder;
// use treebuilder::Treebuilder;

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


fn main() {
    //env::set_var("RUST_BACKTRACE", "1");

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.01, 0.01, 0.1))) // Background 2 Darkblu
        .add_systems(Startup, setup)
        .add_systems(Update, (bevy::window::close_on_esc, process_inputs_system, animate_light_direction, update_scale, pick_node))
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

fn count_entities(all_entities: Query<()>) {
    dbg!(all_entities.iter().count());
}

/// This system prints out all mouse events as they come in
fn process_inputs_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut q_transform: Query<&mut Transform, With<Cam>>,
    mut q_cam: Query<&mut Cam>,  
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut q_pp: Query<&mut Projection, With<Cam>>,

    // mut mouse_button_input_events: EventReader<MouseButtonInput>,
    // mut cursor_moved_events: EventReader<CursorMoved>,
    // mut mouse_wheel_events: EventReader<MouseWheel>,
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
            if keys.pressed(KeyCode::Space) {
                let mut i = 1;
                while (1. - cam.fov * 10.0_f32.powf(i as f32)) >= 0. {
                    i += 1;
                }
                cam.fov -= 0.1_f32.powf(i as f32);

            }
            if keys.pressed(KeyCode::AltLeft) {
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
                cam.speed += 0.01;
            }
            if keys.pressed(KeyCode::ControlLeft) {
                cam.speed -= 0.01;

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

    // for event in mouse_button_input_events.iter() {
    //     info!("{:?}", event);
    // }        

    // for event in cursor_moved_events.iter() {
    //     info!("{:?}", event);
    // }

    // for event in mouse_wheel_events.iter() {
    //     info!("{:?}", event);
    // }
    //println!("cnt: {}",cnt);
}
enum GenerationType {
    Cone, 
    Tree, 
}
/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    asset_server: Res<AssetServer>,
    window: Query<&Window>,
) {

    // Mesh Transmutation Experiment Spawning ///////////////////////////////////////////////////////
    let text_mesh;
    let space_mesh;
    let line_mesh: Mesh;

    // (Todo:) No slash at the end of path string "/", lets the root branch go one sibling stock higher

    // (text_mesh, space_mesh, line_mesh) = generator::walk_path_to_mesh("/sys/module", generator::GenerationType::Branch, 4,true, true);
    // (text_mesh, space_mesh, line_mesh) = generator::walk_path_to_mesh("/home/nom/z/cataclysmdda-0.I/data", generator::GenerationType::Branch, 10, true, true);
    // (text_mesh, space_mesh, line_mesh) = generator::walk_path_to_mesh("/run", generator::GenerationType::Branch, 20, true, true);
    // (text_mesh, space_mesh, line_mesh) = generator::walk_path_to_mesh("./TestTree", generator::GenerationType::Branch, 20, true, true);
    (text_mesh, space_mesh, line_mesh) = generator::walk_path_to_mesh("/sys", generator::GenerationType::Branch, 100, false, false);
    
    // // Textmesh
    // let scalef = 1.; 
    // commands.spawn((PbrBundle {
    //     // mesh: meshes.add(generator::generate_space_mesh()),
    //     mesh: meshes.add(text_mesh),
    //     // material: materials.add(Color::rgb(0.6, 0.3, 0.1).into()),
    //     material: materials.add(StandardMaterial {
    //         // base_color_texture: Some(asset_server.load("lettersheetEdges.png")),
    //         base_color_texture: Some(asset_server.load("branchorange.png")),
    //         ..default()
    //     }),
    //     transform: Transform::from_scale(Vec3{x:scalef,y:scalef,z:scalef}),
    //     ..default()
    //     },
    //     TreeMeshMarker,)
    //     );

    // // Spacemesh
    // commands.spawn((PbrBundle {
    //     mesh: meshes.add(space_mesh),
    //     material: materials.add(StandardMaterial {
    //         // base_color_texture: Some(asset_server.load("lettersheetEdges.png")),
    //         base_color_texture: Some(asset_server.load("branchorange.png")),
    //         ..default()
    //     }),
    //     transform: Transform::from_scale(Vec3{x:scalef,y:scalef,z:scalef}),
    //     ..default()
    //     },
    //     TreeMeshMarker,)
    //     );

    // // Linemesh
    // commands.spawn((PbrBundle {
    //     mesh: meshes.add(line_mesh),

    //     material: materials.add(
    //         Color::rgba(16., 0., 0., 1.0),
    //     ),
    //     ..Default::default()

    //     },
    //     TreeMeshMarker,
    //     RenderLayers::layer(0),
    //     )
    //     );


    // let scalef = 1.0; 
    // commands.spawn((PbrBundle {
    //     // mesh: meshes.add(generator::generate_space_mesh()),
    //     mesh: meshes.add(generator::generate_text_mesh("/dev/")),
    //     // material: materials.add(Color::rgb(0.6, 0.3, 0.1).into()),
    //     material: materials.add(StandardMaterial {
    //         // base_color_texture: Some(asset_server.load("lettersheetEdges.png")),
    //         base_color_texture: Some(asset_server.load("branchorange.png")),
    //         ..default()
    //     }),
    //     transform: Transform::from_scale(Vec3{x:scalef,y:scalef,z:scalef}),
    //     ..default()
    //     },
    //     TreeMeshMarker,)
    //     );

    // Default Spawn of Scene Spawning ///////////////////////////////////////////////////////

    // plane
    commands.spawn(PbrBundle {

        mesh: meshes.add(Circle::new(500.)),
        material: materials.add(Color::rgb(0.5, 0.4, 0.5)),
        transform: Transform::from_rotation(Quat::from_rotation_x(-PI/2.)),
        ..default()
    });
    
    // cubes
    // for i in 1..1000
    // {
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    //     transform: Transform::from_xyz(0.0, (5*i) as f32, 0.0),
    //     ..default()
    // });
    // }   

    // Pick test sphere
        // commands.spawn(PbrBundle {
        // mesh: meshes.add(Mesh::from(Sphere { radius: 10. })),
        // material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
        // transform: Transform::from_xyz(0.0, (5) as f32, 0.0),
        // ..default()
        // });

    // // // Point light, torchlike
    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 10000.0,
    //         shadows_enabled: true,
    //         range: 10000.0,
    //         color: Color::Rgba {
    //                 red: 120.0,
    //                 green: 100.0,
    //                 blue: 0.0,
    //                 alpha: 255.0,
    //             },
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(0.0, 20.0, 0.0) * Transform::from_rotation(Quat::from_rotation_x(90.) ),
    //     ..default()
    // });

    // DisplayTextBundle
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "_",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                font: asset_server.load("fonts/Roboto-Regular.ttf"),
                font_size: 20.0,
                ..default()
            },
        ) // Set the justification of the Text
        .with_text_justify(JustifyText::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }),
        DisplayPathText
    )); 

    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "^",
            TextStyle {
                // This font is loaded and will be used instead of the default font.
                font: asset_server.load("fonts/Roboto-Regular.ttf"),
                font_size: 20.0,
                color: Color::rgba(0.7,0.6,0.6,0.5),
                ..default()
            },
        ) // Set the justification of the Text
        .with_text_justify(JustifyText::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(window.single().resolution.height()/2.),
            left: Val::Px(window.single().resolution.width()/2.),
            ..default()
        }),
    )); 

    // Directional Light, Sunlike
    commands.spawn((DirectionalLightBundle {
        directional_light: DirectionalLight {
                        color: Color::Rgba {
                        red: 0.7,
                        green: 0.4,
                        blue: 0.1,
                        alpha:0.1,
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.,200.,0.)*Transform::from_rotation(Quat::from_rotation_x(-90.)),
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
            fov: (90.0 / 360.0) * (std::f32::consts::PI * 2.0),
            aspect_ratio: 1.0,
            ..default()
        }.into(),
        ..default()
    },
        Cam {yaw: 0., pitch: 0., fov: 1.0, speed:0.2, pos: Vec3::ZERO, rot: Quat::from_xyzw(0.0, 0.0, 0.0, 1.0)},
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

// --- // --- // Utils \\ --- \\ --- \\

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
        //transform.rotate_y(time.delta_seconds() * 0.5);
    }
}

fn update_scale(
    keys: Res<ButtonInput<KeyCode>>,
    mut tree: Query<(&mut Transform, &TreeMeshMarker)>,
    mut q_cam: Query<&mut Cam>,

    // mut tree_bounds_data: Option<ResMut<&mut database::Tree>>,
    // mut tree_bounds_data: Query<&mut database::Tree>,
    // mut tree_data: Option<&mut ResMut<database::Tree>>,
    mut tree_data: Option<ResMut<database::Tree>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

)
{
    let mut cam = q_cam.single_mut();

    // Scale 
    if keys.pressed(KeyCode::Digit1) {
        for (mut transform, cube) in &mut tree {
            // transform.translation = -cam.pos * transform.scale;
            transform.scale *= Vec3{x: 0.9,y:0.9,z: 0.9};
            
            // transform.rotate(Quat::from_rotation_y(0.05));
            // transform.translation += Vec3{x: 0.,y:0.,z: 0.};     


            // if let Some(&mut tree_data) = &mut tree_bounds_data {
                // for (i, mut branch ) in tree_data.bounds.into_iter().enumerate(){
                //     branch.center *= transform.scale.y;
                // }
            // }
            // if let Some(tree_data_) = tree_data.as_ref(){
            // for i in 0..tree_data_.bounds.len() - 1 {
            //     tree_data_.as_mut().bounds[i].center *= transform.scale.y;
            //     tree_data_.as_mut().bounds[i].sphere.radius *= transform.scale.y;
            // }
            // }

        }
    }

    if keys.just_released(KeyCode::Digit1) {
        for (transform, cube) in &mut tree {
        if let Some(tree_data) = &mut tree_data {
            println!("Radius: {:?}", tree_data.bounds[2].sphere.radius);

            for i in 0..tree_data.bounds.len(){
                tree_data.bounds[i].center = tree_data.branches[i].transform.transform_point(Vec3::splat(0.));
                tree_data.bounds[i].center *= transform.scale;
                tree_data.bounds[i].sphere.radius = transform.scale.y + transform.scale.y*0.2;

                // commands.spawn(PbrBundle {
                //     mesh: meshes.add(Mesh::from(bevy::math::primitives::Cuboid { half_size: Vec3::splat(tree_data.bounds[i].sphere.radius) })),
                //     material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
                //     transform: Transform::from_xyz(  tree_data.bounds[i].center.x, tree_data.bounds[i].center.y, tree_data.bounds[i].center.z ),
                //     ..default()
                //     },);
            
            }
            

            // for branch in tree_data.bounds.clone().into_iter(){
            //     // println!("akadadada {:?}", branch.center);
            //     commands.spawn((PbrBundle {
            //     mesh: meshes.add(Mesh::from(bevy::math::primitives::Sphere { radius: branch.sphere.radius })),
            //     material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
            //     transform: Transform::from_xyz(  branch.center.x, branch.center.y, branch.center.z ),
            //     ..default()
            //     },
            //     TreeMeshMarker));
            }
        }
    }
    
    
    
    if keys.pressed(KeyCode::Digit2) {
        for (mut transform, cube) in &mut tree {
            // transform.translation = -cam.pos * transform.scale;
            transform.scale *= Vec3{x: 1.1,y:1.1,z: 1.1};
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


    // Fine scale
    if keys.pressed(KeyCode::Digit3) {
        for (mut transform, cube) in &mut tree {
            transform.scale *= Vec3{x: 0.99,y:0.99,z: 0.99};
            // transform.translate_around(Vec3{x: 0.,y:20.,z: -20.}, Quat::from_rotation_y(0.1));
            // transform.translate_around(Vec3{x: 0.,y:20.,z: -20.}, Quat::from_rotation_y(0.1));

        }
    }
    if keys.pressed(KeyCode::Digit4) {
        for (mut transform, cube) in &mut tree {
            transform.scale *= Vec3{x: 1.01,y:1.01,z: 1.01};
        }
    }

    if keys.pressed(KeyCode::Digit5) {
        for (mut transform, cube) in &mut tree {
            // transform.scale *= Vec3{x: 1.01,y:1.01,z: 1.01};
            transform.rotate(Quat::from_rotation_y(0.05));
        }
    }

}
fn pick_node(
    buttons: Res<ButtonInput<MouseButton>>,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut tree: Query<(&mut Mesh, &TreeMeshMarker)>,

    // q: Query<&Camera>,
    mut q_cam: Query<&mut Cam>,  
    // mut cam_transform: Query<&mut Transform, With<Cam>>,
    // mut q_pp: Query<&mut Camera, With<Cam>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Cam>>,
    q_window: Query<&Window>,
    asset_server: Res<AssetServer>,
    mut display_text_query: Query<&mut Text, With<DisplayPathText>>,



    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    mut tree_data: Option<ResMut<database::Tree>>,
    mut tree_transform: Query<(&mut Transform, &TreeMeshMarker)>,

){



    if buttons.just_released(MouseButton::Right) {

        let mut baum = database::Tree::new();
        // baum.construct("./TestTree/Tree".to_string()); // No end "/" allowed
        // baum.construct("/home/nom/code/rust/storytree".to_string()); // No end "/" allowed
        // baum.construct("/home/nom/z/cata01_02".to_string()); // No end "/" allowed
        // baum.construct("/sys".to_string()); // No end "/" allowed
        // baum.construct("/".to_string()); // No end "/" allowed
        baum.construct("/sys".to_string()); // No end "/" allowed


        // println!("{:?}",baum.branch);
        // for i in 0..2 f{
        //     println!("{:?}",baum.branches[i].name);
        //     println!("{:?}",baum.branches[i].num_of_children);
        // }
        
        commands.spawn((PbrBundle {
            mesh: meshes.add(baum.grow()
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
            mesh: meshes.add(baum.mesh_nodes()
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
    
            commands.insert_resource(baum.clone());

        // Bounds drawing
        // if let Some(tree_data) = &mut tree_data {

        //     for branch in tree_data.bounds.clone().into_iter(){
        //         // println!("akadadada {:?}", branch.center);
        //         commands.spawn((PbrBundle {
        //         mesh: meshes.add(Mesh::from(bevy::math::primitives::Sphere { radius: branch.sphere.radius })),
        //         material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
        //         transform: Transform::from_xyz(  branch.center.x, branch.center.y, branch.center.z ),
        //         ..default()
        //         },
        //         TreeMeshMarker));
        //     }
        // }
        



        // Left button was pressed
        // println!("Ooioioio");

        // Build ray from curor or screencenter position with camera transformation? , iterate over all points and check for hits
        // Get the distance of an intersection with a BoundingSphere, if any.
        // sphere_intersection_at(&self, sphere: &BoundingSphere);

        // pub struct Ray3d {
        //     pub origin: Vec3,
        //     pub direction: Direction3d,
        // }
        // for (mut mesh, cube) in &mut meshes {

        // }
    }   

    if buttons.just_released(MouseButton::Left) {
        let window = q_window.single();
        let (camera, camera_transform) = q_camera.single();
        
        if false {
            for (transform, marker) in &mut tree_transform {
                if let Some(cam_ray) = window
                    .cursor_position()
                    // .and_then(|cursor| camera.viewport_to_world(camera_transform, Vec2::new( window.resolution.width()/2., window.resolution.height()/2.))) //cursor))
                    .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))

                    // .and_then(|cursor| camera.viewport_to_world(&camera_transform.mul_transform(Transform::from_scale(Vec3::splat(1./transform.scale.y))), Vec2::new( window.resolution.width()/2., window.resolution.height()/2.))) //cursor))
                {
                    // eprintln!("World coords: {}/{:?}", world_position.origin, world_position.direction);

                    if let Some(tree_data) = &mut tree_data {
                        let ray_cast = RayCast3d::from_ray(cam_ray, 10000.);

                        for (i, branch, ) in tree_data.bounds.clone().into_iter().enumerate(){
                            let cast_result = ray_cast.intersects(&branch);//BoundingSphereCast::from_ray(branch, world_position, 10000.);
                            if cast_result == true {
                                
                                println!("#{} CastResult: {:?} Path: {:?}", i, cast_result, tree_data.branches[i].name);

                                let mut text = display_text_query.single_mut();
                                text.as_mut().sections[0].value = tree_data.branches[i].name.clone();
                                
                            }
                        }
                        println!("Closed \n");
                    }
                }
            }
        }
        else {
            if let Some(tree_data) = &mut tree_data {
                // let ray_cast = RayCast3d::from_ray(cam_ray, 10000.);
                let bound_sphere = BoundingSphere::new(camera_transform.transform_point(Vec3::splat(0.)), 100.0);
                for (i, branch, ) in tree_data.bounds.clone().into_iter().enumerate(){
                    let cast_result = bound_sphere.intersects(&branch);//BoundingSphereCast::from_ray(branch, world_position, 10000.);
                    if cast_result == true {

                        println!("#{} CastResult: {:?} Path: {:?}", i, cast_result, tree_data.branches[i].name);

                        let mut text = display_text_query.single_mut();
                        text.as_mut().sections[0].value = tree_data.branches[i].name.clone();
                        
                    }
                }
                println!("Closed \n");
            }
        }
    }
    
    // if buttons.just_released(MouseButton::Left) {
    //     // Left Button was released
    // }
    if buttons.pressed(MouseButton::Right) {
        // Right Button is being held down
    }
    // we can check multiple at once with `.any_*`
    if buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        // Either the left or the right button was just pressed
    }
}