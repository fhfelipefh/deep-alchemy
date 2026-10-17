use bevy::prelude::*;
use avian3d::prelude::*;
use crate::alchemy::AlchemicalIngredient;
use super::camera::PlayerCamera;

#[derive(Component)]
#[allow(dead_code)]
pub struct Interactable {
    pub label: &'static str,
}

#[derive(Component)]
#[allow(dead_code)]
pub struct GrabbedProp {
    pub target_distance: f32,
}

#[derive(Resource, Default)]
pub struct InteractionState {
    pub hovered_entity: Option<Entity>,
}

pub fn player_raycast_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    spatial_query: SpatialQuery,
    camera_query: Query<(&GlobalTransform, &Camera), With<PlayerCamera>>,
    mut interactables: Query<(Entity, &GlobalTransform, Option<&mut AlchemicalIngredient>), With<Interactable>>,
    mut interaction_state: ResMut<InteractionState>,
    mut commands: Commands,
) {
    let Ok((cam_gtf, _)) = camera_query.get_single() else {
        return;
    };

    let ray_pos = cam_gtf.translation();
    let ray_dir = cam_gtf.forward();
    let max_distance = 3.5;

    // Check hit against physical colliders
    let hit = spatial_query.cast_ray(
        ray_pos,
        Dir3::new(*ray_dir).unwrap_or(Dir3::NEG_Z),
        max_distance,
        true,
        &SpatialQueryFilter::default(),
    );

    let mut found_target = None;

    if let Some(hit_data) = hit {
        if interactables.contains(hit_data.entity) {
            found_target = Some(hit_data.entity);
        }
    }

    // Also check proximity if collider not hit
    if found_target.is_none() {
        for (entity, gtf, _) in interactables.iter() {
            let to_target = gtf.translation() - ray_pos;
            let dist = to_target.length();
            if dist < max_distance {
                let dot = to_target.normalize().dot(*ray_dir);
                if dot > 0.94 {
                    found_target = Some(entity);
                    break;
                }
            }
        }
    }

    interaction_state.hovered_entity = found_target;

    // Press 'E' to interact
    if keys.just_pressed(KeyCode::KeyE) {
        if let Some(target) = found_target {
            if let Ok((_, _, maybe_ingredient)) = interactables.get_mut(target) {
                if let Some(mut ingredient) = maybe_ingredient {
                    if !ingredient.harvested {
                        ingredient.harvested = true;
                        info!("Harvested: {}", ingredient.name);
                        commands.entity(target).despawn_recursive();
                    }
                }
            }
        }
    }
}

pub fn player_physics_grab(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    interaction_state: Res<InteractionState>,
    mut grabbed_query: Query<(Entity, &mut Transform, &GrabbedProp)>,
    mut commands: Commands,
) {
    let Ok(cam_gtf) = camera_query.get_single() else {
        return;
    };

    // Right click initiates grab
    if mouse_buttons.just_pressed(MouseButton::Right) {
        if let Some(target) = interaction_state.hovered_entity {
            commands.entity(target).insert((
                GrabbedProp { target_distance: 2.2 },
                RigidBody::Kinematic, // Temporarily kinematic while held
            ));
        }
    }

    // While holding, update position in front of camera
    if mouse_buttons.pressed(MouseButton::Right) {
        let hold_point = cam_gtf.translation() + *cam_gtf.forward() * 2.2;
        for (_, mut tf, _) in grabbed_query.iter_mut() {
            tf.translation = tf.translation.lerp(hold_point, 0.2);
        }
    }

    // Release grab
    if mouse_buttons.just_released(MouseButton::Right) {
        for (entity, _, _) in grabbed_query.iter() {
            commands.entity(entity).remove::<GrabbedProp>();
            commands.entity(entity).insert(RigidBody::Dynamic);
        }
    }
}
