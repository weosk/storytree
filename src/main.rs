//! A simple 3D scene with light shining over a cube sitting on a plane.

use std::{env, f32::consts::PI, fs, iter::Map};

use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel}, 
    math::*, pbr::wireframe::WireframeConfig, 
    prelude::*, 
    render::{camera::ScalingMode, mesh::{self, Indices, PrimitiveTopology}}
};

use walkdir::WalkDir;

// mod textgenerator;

// mod treedata;
// use treedata::Treedata;

mod generator;
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
struct treemeshmarker;

fn main() {
    //env::set_var("RUST_BACKTRACE", "1");

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.01, 0.01, 0.1))) // Background 2 Darkblu
        .add_systems(Startup, setup)
        .add_systems(Update, (bevy::window::close_on_esc, process_inputs_system, animate_light_direction, update_scale))

        .insert_resource(AmbientLight {
            color: Color::Rgba {
                red: 0.95,
                green: 0.3,
                blue: 0.0,
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
    keys: Res<Input<KeyCode>>,
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
    for event in mouse_motion_events.iter() {
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
            if keys.pressed(KeyCode::W) {
                delta.z -= cam.speed;
            }
            if keys.pressed(KeyCode::S) {
                delta.z += cam.speed;
            }
            if keys.pressed(KeyCode::A) {
                delta.x -= cam.speed;
            }
            if keys.pressed(KeyCode::D) {
                delta.x += cam.speed;
            }
            if keys.pressed(KeyCode::Q) {
                delta.y -= cam.speed;
            }
            if keys.pressed(KeyCode::E) {
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
    (text_mesh, space_mesh, line_mesh) = generator::walk_path_to_mesh("/", generator::GenerationType::Branch, 100, false, false);

    // Textmesh

    let scalef = 1.0; 
    commands.spawn((PbrBundle {
        // mesh: meshes.add(generator::generate_space_mesh()),
        mesh: meshes.add(text_mesh),
        // material: materials.add(Color::rgb(0.6, 0.3, 0.1).into()),
        material: materials.add(StandardMaterial {
            // base_color_texture: Some(asset_server.load("lettersheetEdges.png")),
            base_color_texture: Some(asset_server.load("branchorange.png")),
            ..default()
        }),
        transform: Transform::from_scale(Vec3{x:scalef,y:scalef,z:scalef}),
        ..default()
        },
        treemeshmarker,)
        );

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
    //     treemeshmarker,)
    //     );

        // Spacemesh
        commands.spawn((PbrBundle {
            mesh: meshes.add(space_mesh),
            material: materials.add(StandardMaterial {
                // base_color_texture: Some(asset_server.load("lettersheetEdges.png")),
                base_color_texture: Some(asset_server.load("branchorange.png")),
                ..default()
            }),
            transform: Transform::from_scale(Vec3{x:scalef,y:scalef,z:scalef}),
            ..default()
            },
            treemeshmarker,)
            );

        // Linemesh
        commands.spawn((PbrBundle {
            mesh: meshes.add(line_mesh),
            material: materials.add(Color::rgba(0.7, 0.6, 0.2, 0.32).into()),
            transform: Transform::from_scale(Vec3{x:scalef,y:scalef,z:scalef}),
            ..default()
            },
            treemeshmarker,)
            );

    // Default Spawn of Scene Spawning ///////////////////////////////////////////////////////

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Circle::new(10.).into()),
        material: materials.add(Color::rgb(0.2, 0.4, 0.4).into()),
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

    // Directional Light, Sunlike
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
                        color: Color::Rgba {
                        red: 0.7,
                        green: 0.4,
                        blue: 0.1,
                        alpha:1.0,
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.,200.,0.)*Transform::from_rotation(Quat::from_rotation_x(-90.)),
        ..default()
    });

    let x = 1.0;
    let y = 1.0;
    let z = 2.0;
    let w = 1.0;

    // Working classic cam
    // commands.spawn((Camera3dBundle {
    //     transform: Transform::from_xyz(0., 10., 40.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     projection: PerspectiveProjection {
    //         fov: (90.0 / 360.0) * (std::f32::consts::PI * 2.0),
    //         aspect_ratio: 1.0,
    //         ..default()
    //     }.into(),
    //     ..default()
    // },
    //     Cam {yaw: 0., pitch: 0., fov: 1.0, speed:0.2, pos: Vec3::ZERO, rot: Quat::from_xyzw(0.0, 0.0, 0.0, 1.0)},
    // ));

    // new 3D orthographic camera
    commands.spawn((Camera3dBundle {
        projection: OrthographicProjection {
            scale: 0.1,
            //scaling_mode: ScalingMode::FixedVertical(15.0),
            ..default()
        }.into(),
        ..default()
    },
            Cam {yaw: 0., pitch: 0., fov: 1.0, speed:0.2, pos: Vec3::ZERO, rot: Quat::from_xyzw(0.0, 0.0, 0.0, 1.0)},
    ));
    
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
    keys: Res<Input<KeyCode>>,
    mut tree: Query<(&mut Transform, &treemeshmarker)>,
    mut q_cam: Query<&mut Cam>
)

{
    let mut cam = q_cam.single_mut();

    // Scale 
    if keys.pressed(KeyCode::Key1) {
        for (mut transform, cube) in &mut tree {
            // transform.translation = -cam.pos * transform.scale;
            transform.scale *= Vec3{x: 0.9,y:0.9,z: 0.9};
            
            // transform.rotate(Quat::from_rotation_y(0.05));
            // transform.translation += Vec3{x: 0.,y:0.,z: 0.};            
        }
    }
    // if keys.just_released(KeyCode::Key1) {
    //     for (mut transform, cube) in &mut tree {
    //         // transform.translation =  cam.pos * transform.scale;          
    //     }
    // }
    if keys.pressed(KeyCode::Key2) {
        for (mut transform, cube) in &mut tree {

            // transform.translation = -cam.pos * transform.scale;
            transform.scale *= Vec3{x: 1.1,y:1.1,z: 1.1};
        }
    }
    // if keys.just_released(KeyCode::Key2) {
    //     for (mut transform, cube) in &mut tree {

    //         // transform.translation = - cam.pos * transform.scale;

    //     }
    // }

    // Fine scale
    if keys.pressed(KeyCode::Key3) {
        for (mut transform, cube) in &mut tree {
            transform.scale *= Vec3{x: 0.99,y:0.99,z: 0.99};
            // transform.translate_around(Vec3{x: 0.,y:20.,z: -20.}, Quat::from_rotation_y(0.1));
            // transform.translate_around(Vec3{x: 0.,y:20.,z: -20.}, Quat::from_rotation_y(0.1));

        }
    }
    if keys.pressed(KeyCode::Key4) {
        for (mut transform, cube) in &mut tree {
            transform.scale *= Vec3{x: 1.01,y:1.01,z: 1.01};
        }
    }

}
