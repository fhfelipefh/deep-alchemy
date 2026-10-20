use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
};
use crate::dungeon::modular_kit::LakeWaterPool;
use crate::player::camera::PlayerCamera;
use super::water_material::WaterMaterial;

#[derive(Resource, Clone, Debug)]
pub struct PlanarReflectionTexture {
    pub texture: Handle<Image>,
    pub water_height: f32,
}

#[derive(Component, Debug, Default)]
pub struct PlanarReflectionCamera;

pub struct PlanarReflectionPlugin;

impl Plugin for PlanarReflectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_planar_reflection);
        app.add_systems(
            Update,
            (
                update_planar_reflection_camera,
                sync_planar_reflection_to_water,
            ),
        );
    }
}

pub fn create_planar_reflection_image(width: u32, height: u32) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[6, 12, 22, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    image
}

pub fn setup_planar_reflection(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let texture = images.add(create_planar_reflection_image(512, 512));

    commands.insert_resource(PlanarReflectionTexture {
        texture: texture.clone(),
        water_height: 0.18,
    });

    commands.spawn((
        Name::new("PlanarReflectionCamera"),
        Camera3d::default(),
        Camera {
            order: -1,
            target: RenderTarget::Image(texture.into()),
            clear_color: ClearColorConfig::Custom(Color::srgba(0.015, 0.035, 0.07, 1.0)),
            ..default()
        },
        PlanarReflectionCamera,
        RenderLayers::from_layers(&[0, 1]),
        Transform::from_xyz(0.0, -1.0, 0.0),
    ));
}

pub fn update_planar_reflection_camera(
    player_cameras: Query<&GlobalTransform, (With<PlayerCamera>, Without<PlanarReflectionCamera>)>,
    mut refl_cameras: Query<&mut Transform, With<PlanarReflectionCamera>>,
    refl_res: Option<Res<PlanarReflectionTexture>>,
) {
    let Some(refl_res) = refl_res else { return; };
    let Ok(player_cam) = player_cameras.get_single() else { return; };
    let Ok(mut refl_transform) = refl_cameras.get_single_mut() else { return; };

    let cam_trans = player_cam.translation();
    let cam_rot = player_cam.compute_transform().rotation;
    let water_y = refl_res.water_height;

    let refl_pos = Vec3::new(cam_trans.x, 2.0 * water_y - cam_trans.y, cam_trans.z);
    let (yaw, pitch, roll) = cam_rot.to_euler(EulerRot::YXZ);
    let refl_rot = Quat::from_euler(EulerRot::YXZ, yaw, -pitch, -roll);

    refl_transform.translation = refl_pos;
    refl_transform.rotation = refl_rot;
}

pub fn sync_planar_reflection_to_water(
    water_pools: Query<&MeshMaterial3d<WaterMaterial>, With<LakeWaterPool>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    refl_res: Option<Res<PlanarReflectionTexture>>,
) {
    let Some(refl_res) = refl_res else { return; };
    for mat_handle in water_pools.iter() {
        if let Some(mat) = water_materials.get_mut(mat_handle) {
            if mat.reflection_texture.is_none() {
                mat.reflection_texture = Some(refl_res.texture.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planar_reflection_texture_resource_and_height() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<Image>();
        app.add_systems(Startup, setup_planar_reflection);
        app.update();

        let refl_res = app.world().get_resource::<PlanarReflectionTexture>();
        assert!(refl_res.is_some(), "PlanarReflectionTexture must be initialized");
        let res = refl_res.unwrap();
        assert_eq!(res.water_height, 0.18, "Water height should default to 0.18");

        let mut cameras = app.world_mut().query_filtered::<&Camera, With<PlanarReflectionCamera>>();
        assert_eq!(cameras.iter(app.world()).count(), 1, "Should spawn 1 PlanarReflectionCamera");
    }

    #[test]
    fn test_planar_reflection_camera_transform_inversion() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<Image>();

        let image_handle = {
            let mut images = app.world_mut().resource_mut::<Assets<Image>>();
            images.add(create_planar_reflection_image(512, 512))
        };

        app.insert_resource(PlanarReflectionTexture {
            texture: image_handle,
            water_height: 0.18,
        });

        let player_cam_tf = Transform::from_xyz(1.5, 1.78, -3.2)
            .with_rotation(Quat::from_euler(EulerRot::YXZ, 0.45, -0.30, 0.05));

        app.world_mut().spawn((
            PlayerCamera,
            player_cam_tf,
            GlobalTransform::from(player_cam_tf),
        ));

        let refl_cam = app.world_mut().spawn((
            PlanarReflectionCamera,
            Transform::default(),
        )).id();

        app.add_systems(Update, update_planar_reflection_camera);
        app.update();

        let refl_tf = app.world().get::<Transform>(refl_cam).unwrap();
        let expected_y = 2.0 * 0.18 - 1.78;
        assert!((refl_tf.translation.y - expected_y).abs() < 1e-4, "Reflected Y should be mirrored across 0.18");
        assert_eq!(refl_tf.translation.x, 1.5);
        assert_eq!(refl_tf.translation.z, -3.2);

        let (yaw, pitch, roll) = refl_tf.rotation.to_euler(EulerRot::YXZ);
        assert!((yaw - 0.45).abs() < 1e-4, "Yaw must match player look");
        assert!((pitch - 0.30).abs() < 1e-4, "Pitch must be inverted");
        assert!((roll - (-0.05)).abs() < 1e-4, "Roll must be inverted");
    }

    #[test]
    fn test_sync_planar_reflection_to_water_pool() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<Image>();
        app.init_asset::<WaterMaterial>();

        let image_handle = {
            let mut images = app.world_mut().resource_mut::<Assets<Image>>();
            images.add(create_planar_reflection_image(512, 512))
        };

        app.insert_resource(PlanarReflectionTexture {
            texture: image_handle.clone(),
            water_height: 0.18,
        });

        let water_mat_handle = {
            let mut mats = app.world_mut().resource_mut::<Assets<WaterMaterial>>();
            mats.add(WaterMaterial::default())
        };

        app.world_mut().spawn((
            LakeWaterPool,
            MeshMaterial3d(water_mat_handle.clone()),
        ));

        app.add_systems(Update, sync_planar_reflection_to_water);
        app.update();

        let mats = app.world().resource::<Assets<WaterMaterial>>();
        let mat = mats.get(&water_mat_handle).unwrap();
        assert_eq!(mat.reflection_texture, Some(image_handle), "Water material must receive planar reflection texture");
    }
}
