//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    prelude::*,
};
use std::{fs, iter::Map};
use std::env;
use walkdir::WalkDir;

use bevy::render::mesh::{self, PrimitiveTopology, Indices};

#[derive(Component, Debug)]
struct Cam
{
    yaw:   f32,
    pitch: f32,
}

fn main() {
    //env::set_var("RUST_BACKTRACE", "1");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(process_inputs_system)
        .add_system(bevy::window::close_on_esc)
        .run();
}

/// This system prints out all mouse events as they come in
fn process_inputs_system(
    keys: Res<Input<KeyCode>>,
    mut q_transform: Query<&mut Transform, With<Cam>>,
    mut q_cam: Query<&mut Cam>,  
    mut mouse_motion_events: EventReader<MouseMotion>,

    // mut mouse_button_input_events: EventReader<MouseButtonInput>,
    // mut cursor_moved_events: EventReader<CursorMoved>,
    // mut mouse_wheel_events: EventReader<MouseWheel>,
) {
    // let mut cnt: i32 = 0;

    for event in mouse_motion_events.iter() {
        for mut cam in q_cam.iter_mut() {
            cam.yaw   += -event.delta.x; 
            cam.pitch += -event.delta.y; 
        }            
    }

    for mut transform in q_transform.iter_mut() {
        
        let mut temp_transform: Transform = Transform{ translation: transform.translation, rotation: transform.rotation, scale: transform.scale,};

        for cam in q_cam.iter_mut() {
                temp_transform.rotation = Quat::from_rotation_y(cam.yaw * 0.005)* Quat::from_rotation_x(cam.pitch * 0.005)  ;            
        }

        let translation_delta = {
            let mut delta = Vec3::ZERO;
            if keys.pressed(KeyCode::W) {
                delta.z -= 0.1;
            }
            if keys.pressed(KeyCode::S) {
                delta.z += 0.1;
            }
            if keys.pressed(KeyCode::A) {
                delta.x -= 0.1;
            }
            if keys.pressed(KeyCode::D) {
                delta.x += 0.1;
            }
            if keys.pressed(KeyCode::Q) {
                delta.y -= 0.1;
            }
            if keys.pressed(KeyCode::E) {
                delta.y += 0.1;
            }
            delta
        };

        temp_transform.translation += temp_transform.rotation * translation_delta;
        *transform = temp_transform;
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

// #[derive(Deserialize, Debug)] 
// struct Items{
//     list: Vec<Entrys>
// }

    

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}



/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    let mut cnt = 0.0; 
    let mut cntOld = 0.0;
    let mut cntVert = 0.0; 
    let mut scalef = 0.01; 


    for entry in WalkDir::new("/home/ben/projects/rust/storytree/").into_iter().filter_map(|e| e.ok()) {
        //println!("{}", entry.path().display());

        if ( entry.file_type().is_dir()) 
        {
            println!("Dir: {:?}", entry.file_name());
            
            commands.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(0.0, 0.5*cnt, 0.0) * Transform::from_scale(Vec3{x:scalef,y:scalef,z:scalef}),
                ..default()
            });
            
            cnt += 0.01;

        }
        else if ( entry.file_type().is_file() )
        {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(cntVert, 0.5*cnt, 0.0) * Transform::from_scale(Vec3{x:scalef,y:scalef,z:scalef}),
                ..default()
            });
            cntVert += 0.01; 

            if cnt != cntOld 
            {
                cntVert *= -1.0;
                cntOld = cnt; 
            }

        }
        else 
        {
            println!("Not Dir nor File: {:?}", entry.file_type());
        }
    }

    println!("cnt: {}", cnt);

// Mesh Transmutation Experiment Spawning ///////////////////////////////////////////////////////

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut valuevec = vec![[-1., 0., 0.], [0., 1., 0.], [1., 0., 0.]];

    valuevec.push({[-1., 0., 0.]});
    valuevec.push({[0., -1., 0.]});
    valuevec.push({[1., 0., 0.]});

    valuevec.extend(vec![[-0.5, 1.5, 0.], [0., 2., 0.], [0.5, 1.5, 0.]]);
    valuevec.extend(vec![[-0.5, 1.5, 0.], [0., 1., 0.], [0.5, 1.5, 0.]]);

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        valuevec,
    );

    // In this example, normals and UVs don't matter,
    // so we just use the same value for all of them
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; 12]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; 12]);
    
    // A triangle using vertices 0, 2, and 1.
    // Note: order matters. [0, 1, 2] will be flipped upside down, and you won't see it from behind!
    mesh.set_indices(Some(mesh::Indices::U32(vec![0, 2, 1, 3, 4, 5, 6, 8, 7, 9, 10, 11])));

    
    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::rgb(0.7, 0.1, 0.3).into()),
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
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // camera
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0., 1., 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },
        Cam {yaw: 0., pitch: 0.},
    ));
}
