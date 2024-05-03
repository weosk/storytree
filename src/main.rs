//! A simple 3D scene with light shining over a cube sitting on a plane.

use std::{env, f32::consts::PI, fs, iter::Map};

use bevy::{
    input::{keyboard::KeyCode, mouse::{MouseButtonInput, MouseMotion, MouseWheel}},
    math::{bounding::{BoundingSphere, BoundingSphereCast, BoundingVolume, IntersectsVolume, RayCast3d}, primitives::Sphere, *}, pbr::{extract_meshes, wireframe::WireframeConfig}, 
    prelude::*,
    render::{camera::ScalingMode, mesh::{self, Indices, PrimitiveTopology}, render_resource::{AsBindGroup, ShaderRef}, view::RenderLayers}
};

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

fn main() {
    //env::set_var("RUST_BACKTRACE", "1");

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.01, 0.01, 0.21))) // Background 2 Darkblu
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
    main_tree.construct("/".to_string()); // No end "/" allowed
    
    commands.spawn((PbrBundle {
        mesh: meshes.add(main_tree.grow()
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
        mesh: meshes.add(Circle::new(500.)),
        material: materials.add(Color::rgb(0.4, 0.3, 0.4)),
        transform: Transform::from_rotation(Quat::from_rotation_x(-PI/2.)),
        ..default()
    });

    // Ui Bundle v , Probably needs Text2dBundle for precise positioning

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
    mut display_text_query: Query<&mut Text, With<DisplayPathText>>,
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
                cam.speed += 0.07;
            }
            if keys.pressed(KeyCode::ControlLeft) {
                cam.speed -= 0.07;

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
            }
            }
        }
    }
    
    if keys.pressed(KeyCode::Digit2) {
        for (mut transform, cube) in &mut tree {
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

    // Node_Picking
    if buttons.just_released(MouseButton::Left) {
        let window = q_window.single();
        let (camera, camera_transform) = q_camera.single();
        
        if true {
            for (transform, marker) in &mut tree {
                if let Some(cam_ray) = window
                    .cursor_position()
                    .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
                {
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
                let bound_sphere = BoundingSphere::new(camera_transform.transform_point(Vec3::splat(0.)), 100.0);
                for (i, branch, ) in tree_data.bounds.clone().into_iter().enumerate(){
                    let cast_result = bound_sphere.intersects(&branch);//BoundingSphereCast::from_ray(branch, world_position, 10000.);
                    if cast_result == true {

                        println!("#{} CastResult: {:?} Path: {:?}", i, cast_result, tree_data.branches[i].name);

                        let mut text = display_text_query.single_mut();
                        text.as_mut().sections[0].value = tree_data.branches[i].name.clone();
                        // text.as_mut().sections[0].style.
                    }
                }
            }
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
        //transform.rotate_y(time.delta_seconds() * 0.5);
    }
}