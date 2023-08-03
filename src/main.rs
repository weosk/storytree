//! A simple 3D scene with light shining over a cube sitting on a plane.
use bevy::math::*;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::World;
use bevy::render::primitives::Frustum;
use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    prelude::*, render::primitives::HalfSpace,
};
use std::{fs, iter::Map};
use std::env;
use walkdir::WalkDir;

use bevy::render::mesh::{self, PrimitiveTopology, Indices};

use bevy::render::camera::CameraProjection;
//use window::Windows;

mod treebuilder;


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

fn main() {
    //env::set_var("RUST_BACKTRACE", "1");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (process_inputs_system, bevy::window::close_on_esc, animate_light_direction))
        .insert_resource(AmbientLight {
            color: Color::Rgba {
                red: 0.95,
                green: 0.3,
                blue: 0.0,
                alpha:1.0,
            },
            brightness: 0.5,})
        .run();
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
            cam.yaw   += -event.delta.x; 
            cam.pitch += -event.delta.y;     
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

            // Manual Zoom
            if keys.pressed(KeyCode::Space) {
                if cam.fov <= 0.005
                {
                    cam.fov -= 0.0005;
                }
                else
                {
                    cam.fov -= 0.005;
                }
            }
            if keys.pressed(KeyCode::AltLeft) {
                cam.fov += 0.005;
            }

            // Adjust Speed
            if keys.pressed(KeyCode::ShiftLeft) {
                cam.speed += 0.01;
            }
            if keys.pressed(KeyCode::ControlLeft) {
                cam.speed -= 0.01;

                treebuilder::print_hello();
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
            *pp = PerspectiveProjection {
                fov: cam.fov,
                aspect_ratio: 1.0,
                ..default()
            }.into()
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


fn print_typename<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
// Mesh Transmutation Experiment Spawning ///////////////////////////////////////////////////////

    // Vertices
    // Plain Data to 4 Triangles, facing back and forth for top and down
    let mut ground_vertices: [[f32; 3]; 12] =   [   [-1., 0., 0.], [0.,  1., 0.], [1., 0., 0.],
                                                [-1., 0., 0.], [0., -1., 0.], [1., 0., 0.],
                                                [-1., 0., 0.], [0.,  1., 0.], [1., 0., 0.], 
                                                [-1., 0., 0.], [0., -1., 0.], [1., 0., 0.] ];

   // Indices, not yet applied, see below
    let ground_indices = [0, 2, 1, 3, 4, 5, 6, 7, 8, 9, 11, 10];
    //println!("Type: {}", ground_indices.type_name());

    // Transformations Matrix
    let tmat:Affine3A = Affine3A::from_translation(Vec3{x:0.0,y:1.0,z:0.0}.into());
    println!("T_Mat: {}, {}", tmat, tmat.translation);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut vertexvec: Vec<[f32; 3]> = vec![];
    let mut indexvec: Vec<u32> = vec![];

    let mut cnt = 0.0; 
    let mut cntOld = 0.0;
    let mut cntVert = 0.0;
    let mut rot = 0.0;

    let mut add_indi: u32 = 0;
    // for entry in WalkDir::new("/home/ben/projects/rust/storytree/").into_iter().filter_map(|e| e.ok()) {
        //println!("{}", entry.path().display());
    for entry in WalkDir::new("/home/ben/projects").into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_dir() 
        {
            // Dive Up and to the side, depending on Directory
            for each in ground_vertices {
                

                vertexvec.push(   ( // Translation * Rotation -> Transform TriangleVertex -> into Vec<[f32; 3]>
                                    Affine3A::from_quat(Quat::from_rotation_y(rot)) *
                                    Affine3A::from_translation(Vec3{x:cntVert*0.02,y:cnt*0.02,z:0.0})
                                  )
                                  .transform_point3( Vec3::from_array(each) ) 
                                  .into()
            );

                // vertexvec.push( Affine3A::from_translation(Vec3{x:cntVert*0.02,y:cnt*0.02,z:0.0}.into()).transform_point3( Vec3::from_array(each)
                // ).into() 
                // );


                // println!("Type: {} Each: {:?} From_Array: {:?}",each.type_name(), each, Vec3::from_array(each).clone());
            }

            // dublicate indizes
            add_indi = 12 * (cnt as u32);
            indexvec.extend(vec![  0+add_indi, 2+add_indi, 1+add_indi, 
                                         3+add_indi, 4+add_indi, 5+add_indi, 
                                         6+add_indi, 7+add_indi, 8+add_indi, 
                                         9+add_indi, 11+add_indi, 10+add_indi]); 

            // println!("Dir: {:?}", entry.path());
            cnt += 1.0;
        }
        else if entry.file_type().is_file() 
        {
            cntVert += 1.0; 
            //println!("Filename: {:?} \n {:?} \n", entry.file_name(), entry.metadata());

            if cnt != cntOld 
            {
                rot=rot+1.0;
                cntVert *= -1.0;
                cntOld = cnt; 
            }
        }
        else if entry.file_type().is_symlink()
        {
            //println!("Symlink: {:?}", entry.file_name());
        }
        else 
        {
            //println!("Not Dir nor File: {:?}", entry.file_name());
        }
    }

    //println!("cnt: {}", cnt);

    // let mut mesh_vec = vec![];
    // mesh_vec.extend(vec![[-1., 0., 0.], [0., 1., 0.], [1., 0., 0.] ]);
    // mesh_vec.extend(vec![[-1., 0., 0.], [0., -1., 0.], [1., 0., 0.]]);
    // mesh_vec.extend(vec![[-1., 0., 0.], [0., 1., 0.], [1., 0., 0.] ]);
    // mesh_vec.extend(vec![[-1., 0., 0.], [0., -1., 0.], [1., 0., 0.]]);

    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; vertexvec.len()]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; vertexvec.len()]);
    
    println!("vertexvecLen: {}", vertexvec.len());

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vertexvec,
    );

    // In this example, normals and UVs don't matter,
    // so we just use the same value for all of them
    // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; 12]);
    // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; 12]);
    
    // // A triangle using vertices 0, 2, and 1.
    // // Note: order matters. [0, 1, 2] will be flipped upside down, and you won't see it from behind!
    // mesh.set_indices(Some(mesh::Indices::U32(vec![0, 2, 1, 3, 4, 5, 6, 7, 8, 9, 11, 10]))); 
                                                //12, 14, 13, 15, 16, 17, 18, 19, 20, 21, 23, 22
                                                //24, 26, 25, 27, 28, 29, 30, 32, 32, 33, 35, 34

    mesh.set_indices(Some(mesh::Indices::U32(indexvec)));
    
    let mut scalef = 1.0; 
    // MyMesh
    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::rgb(0.6, 0.3, 0.1).into()),
        transform: Transform::from_scale(Vec3{x:scalef,y:scalef,z:scalef}),
        ..default()
    });

// Default Spawn of Scene Spawning ///////////////////////////////////////////////////////

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 20., 0.0),
        ..default()
    });

    // // // Point light, torchlike
    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 0.1,
    //         shadows_enabled: true,
    //         range: 10000.0,
            // color: Color::Rgba {
            //         red: 120.0,
            //         green: 100.0,
            //         blue: 0.0,
            //         alpha: 255.0,
            //     },
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(0.0, 8.0, 0.0),
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
        transform: Transform::from_rotation(Quat::from_rotation_x(-90.) ),
        ..default()
    });

    let x = 1.0;
    let y = 1.0;
    let z = 2.0;
    let w = 1.0;
    
    // camera
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0., 10., 40.0).looking_at(Vec3::ZERO, Vec3::Y),
        projection: PerspectiveProjection {
            fov: (90.0 / 360.0) * (std::f32::consts::PI * 2.0),
            aspect_ratio: 1.0,
            ..default()
        }.into(),
        ..default()
    },
        Cam {yaw: 0., pitch: 0., fov: 1.0, speed:0.1, pos: Vec3::ZERO, rot: Quat::from_xyzw(0.0, 0.0, 0.0, 1.0)},
    ));

    // new 3D orthographic camera
    // commands.spawn_bundle(Camera3dBundle {
    //     projection: OrthographicProjection {
    //         scale: 3.0,
    //         scaling_mode: ScalingMode::FixedVertical(5.0),
    //         ..default()
    //     }.into(),
    //     ..default()
    // })

}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
    mut q_cam: Query<&mut Cam>, 
) {
    // println!("Cam: {:?}", q_cam.single_mut().pos);
    for mut transform in &mut query {
        *transform = Transform::from_rotation(q_cam.single_mut().rot) * Transform::from_xyz(0.0, q_cam.single_mut().pos.y+30.0, 0.0);
        //transform.rotate_y(time.delta_seconds() * 0.5);
    }
}