use std::fs;

use bevy::{prelude::*, render::render_resource::PrimitiveTopology};

use rw_rs::bsf::*;

#[derive(Component)]
struct TheMesh;

#[derive(Resource)]
struct MeshIndex(usize);

#[derive(Resource)]
struct Meshes(Vec<Handle<Mesh>>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "DFF Viewer".into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (input_handler, update_mesh))
        .run();
}

fn load_meshes(bsf: &Chunk) -> Vec<Mesh> {
    let mut mesh_vec = Vec::new();

    for geometry_chunk in bsf
        .get_children()
        .iter()
        .find(|e| matches!(e.content, ChunkContent::GeometryList))
        .unwrap()
        .get_children()
    {
        if let ChunkContent::Geometry(geo) = &geometry_chunk.content {
            let topo = if geo.is_tristrip() {
                PrimitiveTopology::TriangleStrip
            } else {
                PrimitiveTopology::TriangleList
            };
            let mut mesh = Mesh::new(topo);
            mesh.set_indices(Some(bevy::render::mesh::Indices::U16(
                geo.triangles
                    .iter()
                    .flat_map(|t| t.as_arr())
                    .collect::<Vec<_>>(),
            )));
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_POSITION,
                geo.vertices.iter().map(|t| t.as_arr()).collect::<Vec<_>>(),
            );
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                geo.normals.iter().map(|t| t.as_arr()).collect::<Vec<_>>(),
            );
            mesh_vec.push(mesh);
        }
    }
    mesh_vec
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let file = fs::read("player.dff").unwrap();
    let (_, bsf) = Chunk::parse(&file).unwrap();

    commands.insert_resource(MeshIndex(0));

    // Create and save a handle to the mesh.
    let cube_mesh_handles: Vec<Handle<Mesh>> = load_meshes(&bsf)
        .into_iter()
        .map(|m| meshes.add(m))
        .collect();

    commands.insert_resource(Meshes(cube_mesh_handles.clone()));

    // Render the mesh with the custom texture using a PbrBundle, add the marker.
    commands.spawn((
        PbrBundle {
            mesh: cube_mesh_handles[0].clone(),
            material: materials.add(StandardMaterial { ..default() }),
            ..default()
        },
        TheMesh,
    ));

    // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
    let camera_and_light_transform =
        Transform::from_xyz(1.8, 1.8, 1.8).looking_at(Vec3::ZERO, Vec3::Y);

    // Camera in 3D space.
    commands.spawn(Camera3dBundle {
        transform: camera_and_light_transform,
        ..default()
    });

    // Light up the scene.
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1000.0,
            range: 100.0,
            ..default()
        },
        transform: camera_and_light_transform,
        ..default()
    });

    commands.spawn(
        TextBundle::from_section(
            "Controls:\nX/Y/Z: Rotate\nR: Reset orientation\n+/-: Show different geometry in dff",
            TextStyle {
                font_size: 20.0,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        }),
    );
}

fn input_handler(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<TheMesh>>,
    mut index: ResMut<MeshIndex>,
    meshes: Res<Meshes>,
    time: Res<Time>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        //let mesh_handle = mesh_query.get_single().expect("Query not successful");
        //let mesh = meshes.get_mut(mesh_handle).unwrap();
        //toggle_texture(mesh);
    }
    if keyboard_input.pressed(KeyCode::X) {
        for mut transform in &mut query {
            transform.rotate_x(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::Y) {
        for mut transform in &mut query {
            transform.rotate_y(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::Z) {
        for mut transform in &mut query {
            transform.rotate_z(time.delta_seconds() / 1.2);
        }
    }
    if keyboard_input.pressed(KeyCode::R) {
        for mut transform in &mut query {
            transform.look_to(Vec3::NEG_Z, Vec3::Y);
        }
    }
    if keyboard_input.just_pressed(KeyCode::Plus) | keyboard_input.just_pressed(KeyCode::NumpadAdd)
    {
        let num_meshes = meshes.0.len();
        if index.0 < num_meshes - 1 {
            index.0 += 1;
        }
    }
    if keyboard_input.just_pressed(KeyCode::Minus)
        | keyboard_input.just_pressed(KeyCode::NumpadSubtract)
        && index.0 > 0
    {
        index.0 -= 1;
    }
}

fn update_mesh(
    mut commands: Commands,
    mesh_query: Query<Entity, With<TheMesh>>,
    index: Res<MeshIndex>,
    meshes: Res<Meshes>,
) {
    if index.is_changed() {
        let new_mesh = meshes.0.get(index.0).unwrap().clone();
        commands.entity(mesh_query.single()).insert(new_mesh);
    }
}
