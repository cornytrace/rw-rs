use std::fs;

use bevy::{prelude::*, render::render_resource::PrimitiveTopology};

use rw_rs::bsf::*;

#[derive(Component)]
struct TheMesh;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, input_handler)
        .run();
}

fn load_mesh(bsf: &BsfChunk) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    /*for geometry_chunk in &bsf
    .children
    .iter()
    .find(|e| e.ty == ChunkType::GeometryList)
    .unwrap()
    .children[1..]*/
    let geometry_chunk = &bsf
        .children
        .iter()
        .find(|e| e.ty == ChunkType::GeometryList)
        .unwrap()
        .children[1];
    {
        if let BsfChunkContent::RpGeometry(geo) = &geometry_chunk.content {
            let topo = if geo.is_tristrip() {
                PrimitiveTopology::TriangleStrip
            } else {
                PrimitiveTopology::TriangleList
            };
            mesh = Mesh::new(topo);
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
            )
        }
    }
    mesh
}

fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let file = fs::read("player.dff").unwrap();
    let (_, bsf) = parse_bsf_chunk(&file).unwrap();

    // Create and save a handle to the mesh.
    let cube_mesh_handle: Handle<Mesh> = meshes.add(load_mesh(&bsf));

    // Render the mesh with the custom texture using a PbrBundle, add the marker.
    commands.spawn((
        PbrBundle {
            mesh: cube_mesh_handle,
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
}

fn input_handler(
    keyboard_input: Res<Input<KeyCode>>,
    mesh_query: Query<&Handle<Mesh>, With<TheMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<&mut Transform, With<TheMesh>>,
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
}
