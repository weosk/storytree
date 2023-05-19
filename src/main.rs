//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    prelude::*,
};

#[derive(Component, Debug)]
struct Cam
{
    yaw:   f32,
    pitch: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(update_cam)
        .add_system(print_mouse_events_system)
        //.add_system(keyboard_input) // may need after keyword
        .run();
}

//mut right_camera: Query<&mut Camera, With<RightCamera>>,

fn update_cam (
    mut cams: Query<&mut Transform, With<Cam>>
    ){
        for mut cam in &mut cams {
            // cam.translation += Vec3{ x:0.0 , y:0.01 , z:0.2 };
            // cam.rotation = Quat::from_rotation_x(cam.translation.y);
            // info!("{:?}",cam);
        }
}

// fn keyboard_input(
//     mut q_transform: Query<&mut Transform, With<Cam>>,
//     mut q_cam: Query<&mut Cam>
//     mut 
// ) {

// }

// fn keyboard_input(
//     keys: Res<Input<KeyCode>>,
// ) {
//     if keys.just_pressed(KeyCode::Space) {
//         println!(" Space was pressed ");
//     }
//     if keys.just_released(KeyCode::LControl) {
//         // Left Ctrl was released
//     }
//     if keys.pressed(KeyCode::W) {
//         // Forward movement
//         transform.translation += rot * Vec3{x:0.,y: 0.,z:-0.1};
//     }
//     // we can check multiple at once with `.any_*`
//     if keys.any_pressed([KeyCode::LShift, KeyCode::RShift]) {
//         // Either the left or right shift are being held down
//     }
//     if keys.any_just_pressed([KeyCode::Delete, KeyCode::Back]) {
//         // Either delete or backspace was just pressed
//     }
// }

/// This system prints out all mouse events as they come in
fn print_mouse_events_system(
    keys: Res<Input<KeyCode>>,
    mut q_transform: Query<&mut Transform, With<Cam>>,
    mut q_cam: Query<&mut Cam>,  
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
) {

    //let transform = q_transform.get_single_mut();
    //let cam = q_cam.get_single_mut();

    for mut transform in q_transform.iter_mut() {
        let rot = transform.rotation;    

        for event in mouse_button_input_events.iter() {
            info!("{:?}", event);
        }

        for event in mouse_motion_events.iter() {

            for mut cam in q_cam.iter_mut() {
                cam.yaw += -event.delta.x; 
                cam.pitch += -event.delta.y; 
                
                //for mut transform in q_transform.iter_mut() {
                transform.rotation = Quat::from_rotation_x(cam.pitch * 0.005) * Quat::from_rotation_y(cam.yaw * 0.005);
                    //println!("{:?}", transform.rotation.    );
                //let rot = transform.rotation;
                    

                    
                    //transform.translation = Vec3{x:0.,y: 0.,z:-0.1} * transform.rotation; 
                //}
            }

            //cam.yaw += -event.delta.x; 
            //cam.pitch += -event.delta.y; 
            // for mut cam in &mut cams {
            //     cam.yaw += -event.delta.x; 
            //     cam.pitch += -event.delta.y; 
            //     //ct.rotation *= Quat::from_rotation_x(-event.delta.y * 0.01);
            //     //cam.mul_transform(transform)
            // }
            // info!("{:?}", event);
        }

        for event in cursor_moved_events.iter() {
            info!("{:?}", event);
        }

        for event in mouse_wheel_events.iter() {
            info!("{:?}", event);
        }

        // Forward movement
        if keys.pressed(KeyCode::W) {    
            transform.translation += rot * Vec3{x:0.,y: 0.,z:-0.1};
        }

        if keys.pressed(KeyCode::S) {    
            transform.translation += rot * Vec3{x:0.,y: 0.,z:0.1};
        }

        if keys.pressed(KeyCode::A) {    
            transform.translation += rot * Vec3{x:-0.1,y: 0.,z:0.};
        }

        if keys.pressed(KeyCode::D) {    
            transform.translation += rot * Vec3{x:0.1,y: 0.,z:0.};
        }

    }

}



/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
