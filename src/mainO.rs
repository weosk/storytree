// use bevy::{prelude::*, math::Vec3A};
// use smooth_bevy_cameras::{LookTransform, LookTransformBundle, LookTransformPlugin, Smoother};

// use bevy::render::mesh::{self, PrimitiveTopology};

// #[derive(Resource)]
// struct Cam {
//     eye: Vec3<>,
//     target: Vec3<>,
//     up: Vec3<>,
//     /*
//     aspect: f32,
//     fovy: f32,1
//     znear: f32,
//     zfar: f32,
//  */
// }

// // impl FeedCam for Camera {
// //     fn feedcam() -> Self{

// //         Camera {}
// //     }

// // }

// fn main() {
//     App::new()
//         .insert_resource(Msaa::Sample4)
//         .insert_resource(Cam{eye: Vec3 { x: 0., y: 2.5, z: 5.0 }, target: Vec3 { x: 0., y: 1., z: 0. }, up: Vec3{ x: 0., y:1., z: 0. }})
//         .add_plugins(DefaultPlugins)
//         // Enables the system that synchronizes your `Transform`s and `LookTransform`s.
//         .add_plugin(LookTransformPlugin)
//         .add_startup_system(setup)
//         .add_system(move_camera_system)
//         .add_system(keyboard_input_system)
//         .run();
// }


// fn setup(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     ) {

//     let eye = Vec3 { x: -2., y: 2.5, z: 5.0 };
//     let target = Vec3::default();
//     //(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y)

//     println!("{}", eye);

//     commands
//         .spawn(LookTransformBundle {
//             transform: LookTransform::new(eye, target, Vec3 { x: 0., y: 1., z: 0. }),
//             smoother: Smoother::new(0.9), // Value between 0.0 and 1.0, higher is smoother.
//         })
//         .insert(Camera3dBundle::default());

//     let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

//     // Positions of the vertices
//     // See https://bevy-cheatbook.github.io/features/coords.html
//     mesh.insert_attribute(
//         Mesh::ATTRIBUTE_POSITION,
//         vec![[0., 0., 0.], [1., 2., 1.], [2., 0., 0.]],
//     );


//     // In this example, normals and UVs don't matter,
//     // so we just use the same value for all of them
//     mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; 3]);
//     mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; 3]);

//     // A triangle using vertices 0, 2, and 1.
//     // Note: order matters. [0, 1, 2] will be flipped upside down, and you won't see it from behind!
//     mesh.set_indices(Some(mesh::Indices::U32(vec![0, 2, 1])));

//     commands.spawn(PbrBundle {
//         mesh: meshes.add(mesh),
//         material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
//         ..default()
//     });

//     commands.spawn(PointLightBundle {
//         point_light: PointLight {
//             intensity: 1500.0,
//             shadows_enabled: true,
//             ..default()
//         },
//         transform: Transform::from_xyz(4.0, 8.0, 4.0),
//         ..default()
//     });
    
    
//     // commands.spawn(Camera3dBundle {
//     //     transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
//     //     ..default()
//     // });
 
// }

// fn move_camera_system(mut cameras: Query<&mut LookTransform>, cam: Res<Cam>) {
//     // Later, another system will update the `Transform` and apply smoothing automatically.
//     for mut c in cameras.iter_mut() {c.eye = cam.eye; c.target = cam.target;}
// }

// fn keyboard_input_system(keyboard_input: Res<Input<KeyCode>>, mut cam :ResMut<Cam>) {
//     if keyboard_input.pressed(KeyCode::A) {
//         cam.target.x += 0.5; 
//     }

//     if keyboard_input.pressed(KeyCode::D) {
//         cam.target.x -= 0.5; 
//     }

//     if keyboard_input.pressed(KeyCode::W) {
//         cam.target.z += 0.5; 
//     }

//     if keyboard_input.pressed(KeyCode::S) {
//         cam.target.z -= 0.5; 
//     }


//     if keyboard_input.pressed(KeyCode::Q) {
//         cam.target.y += 0.5; 
//     }

//     if keyboard_input.pressed(KeyCode::E) {
//         cam.target.y -= 0.5; 
//     }


//     if keyboard_input.pressed(KeyCode::Up) {
//         cam.eye.z += 0.5; 
//     }

//     if keyboard_input.pressed(KeyCode::Down) {
//         cam.eye.z -= 0.5; 
//     }

// }