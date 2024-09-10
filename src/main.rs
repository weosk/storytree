// use nix::unistd::Uid;
// fn main() {
//     if !Uid::effective().is_root() {
//         panic!("You must run this executable with root permissions");
//     }
// } 
use std::time::Instant;
use bevy::time::Time;
use bevy::prelude::*;
use std::f32::consts::PI;
use interactionframework::process_inputs_system;

mod interactionframework;
mod generator;
mod database;
use bevy_fps_counter::{FpsCounter, FpsCounterPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(1.0, 0.92, 0.0))) // Background 2 Darkblu
        .add_systems(Startup, setup)
        .add_systems(Update, (bevy::window::close_on_esc, process_inputs_system, animate_light_direction))
        // .insert_resource(AmbientLight {color: Color::Rgba {red: 0.95,green: 0.3,blue: 1.0,alpha:1.0,},brightness: 0.5,},)
        // .add_plugins(FrameTimeDiagnosticsPlugin::default())
        // .add_plugins(FrameTimeDiagnosticsPlugin::default())

        .add_plugins(FpsCounterPlugin)

        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,

    // For Spawning trees in Setup
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    time: Res<Time>,
    mut diags_state: ResMut<FpsCounter>
) {
    diags_state.enable();
    // Spwan MemoryTree
    let now = Instant::now();
    interactionframework::spawn_tree("/".to_string(), Vec3 { x: 0., y: 0., z: 0. }, (1.,0.9,0.5,1.1), &mut commands, &mut meshes,&mut materials);
    let elapsed = now.elapsed();
    println!("Elapsed: {:?}",elapsed);

    // for i in (0..5).step_by(1) {

    //     let now = Instant::now();
    //     interactionframework::spawn_generator_tree("/".to_string(), Vec3 { x: i as f32 * 1., y: 0., z: 0. }, &mut commands, &mut meshes,&mut materials, true, true, i);
    //     let elapsed = now.elapsed();
    //     println!("#: {:?} Elapsed: {:?}",i, elapsed);
    // }    

    // Plane
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Circle::new(5000000.)),
    //     material: materials.add(Color::rgb(1., 1., 1.)),
    //     // material: materials.add(Color::rgb(0.4, 0.3, 0.4)),
    //     transform: Transform::from_rotation(Quat::from_rotation_x(-PI/2.)),
    //     ..default()
    // });

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

    // Perspective cam
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0., 11., 47.).looking_at(Vec3::ZERO, Vec3::Y),
        projection: PerspectiveProjection {
            ..default()
        }.into(),
        ..default()
    },
        interactionframework::Cam {yaw: 0., pitch: 0., fov: 1., speed:0.47, pos: Vec3::ZERO, rot: Quat::from_xyzw(0.0, 0.0, 0.0, 1.0)},
    ));

    // v v v Ui Bundle v v v

    // 2D camera to draw glyps upon, overlayed over the 3D view
    commands.spawn(Camera2dBundle{camera: Camera{order:1,..default()},..Default::default()});

    // Ui 
    let font = asset_server.load("fonts/Roboto-Bold.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 18.0,
        color: Color::BLACK,
        ..default()
    };

    let mut cnt = 0;
    for _i in 0..400{
        cnt += 1;
            commands.spawn((
            Text2dBundle {
                text: Text::from_section("", text_style.clone())
                    .with_justify(JustifyText::Center),
                text_anchor: bevy::sprite::Anchor::CenterLeft,
                ..default()
            },
            interactionframework::DisplayPathText,
        ));
    }
    println!("Number of spawned text display buffer: {:?}", cnt);


}

fn _count_entities(all_entities: Query<()>) {
    dbg!(all_entities.iter().count());
}

fn _print_typename<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

// For torchlike experiments
fn animate_light_direction(
    // time: Res<Time>,
    // mut query: Query<&mut Transform, With<DirectionalLight>>,
    // mut q_cam: Query<&mut interactionframework::Cam>, 
) {
    // println!("Cam: {:?}", q_cam.single_mut().pos);
    // for mut transform in &mut query {
        
        // *transform = Transform::from_rotation(q_cam.single_mut().rot) * Transform::from_xyz(0.0, q_cam.single_mut().pos.y+0.0, 0.0);
        // *transform = Transform::from_rotation(q_cam.single_mut().rot) * Transform::from_xyz(q_cam.single_mut().pos.x, q_cam.single_mut().pos.y+0.0, q_cam.single_mut().pos.z);
        // transform.rotate_y(time.delta_seconds() * 0.5);
    // }
}