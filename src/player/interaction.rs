use super::camera::PlayerCamera;
use crate::{alchemy::AlchemicalIngredient, ui::menu::GameState};
use avian3d::prelude::*;
use bevy::prelude::*;

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
    game_state: Option<Res<State<GameState>>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    gamepads: Query<&Gamepad>,
    spatial_query: SpatialQuery,
    camera_query: Query<(&GlobalTransform, &Camera), With<PlayerCamera>>,
    mut interactables: Query<
        (Entity, &GlobalTransform, Option<&mut AlchemicalIngredient>),
        With<Interactable>,
    >,
    mut interaction_state: ResMut<InteractionState>,
    touch_controls: Option<Res<crate::player::touch_controls::TouchControlsState>>,
    mut commands: Commands,
) {
    if let Some(state) = game_state {
        if *state.get() != GameState::InGame {
            interaction_state.hovered_entity = None;
            return;
        }
    }

    let Ok((cam_gtf, _)) = camera_query.get_single() else {
        return;
    };

    let ray_pos = cam_gtf.translation();
    let ray_dir = cam_gtf.forward();
    let max_distance = 3.5;

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

    let interact_pressed = keys.as_ref().map(|k| k.just_pressed(KeyCode::KeyE)).unwrap_or(false)
        || gamepads.iter().any(|g| g.just_pressed(GamepadButton::West))
        || touch_controls.as_ref().map_or(false, |t| t.interact_just_pressed);

    if interact_pressed {
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
    game_state: Option<Res<State<GameState>>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    interaction_state: Res<InteractionState>,
    mut grabbed_query: Query<(Entity, &mut Transform, &GrabbedProp)>,
    mut commands: Commands,
) {
    if let Some(state) = game_state {
        if *state.get() != GameState::InGame {
            return;
        }
    }

    let Ok(cam_gtf) = camera_query.get_single() else {
        return;
    };

    if mouse_buttons.just_pressed(MouseButton::Right) {
        if let Some(target) = interaction_state.hovered_entity {
            commands.entity(target).insert((
                GrabbedProp {
                    target_distance: 2.2,
                },
                RigidBody::Kinematic,
            ));
        }
    }

    if mouse_buttons.pressed(MouseButton::Right) {
        let hold_point = cam_gtf.translation() + *cam_gtf.forward() * 2.2;
        for (_, mut tf, _) in grabbed_query.iter_mut() {
            tf.translation = tf.translation.lerp(hold_point, 0.2);
        }
    }

    if mouse_buttons.just_released(MouseButton::Right) {
        for (entity, _, _) in grabbed_query.iter() {
            commands.entity(entity).remove::<GrabbedProp>();
            commands.entity(entity).insert(RigidBody::Dynamic);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_grab_initiates_and_releases() {
        let mut world = World::new();
        let mut mouse = ButtonInput::<MouseButton>::default();
        mouse.press(MouseButton::Right);
        world.insert_resource(mouse);

        let mut interaction_state = InteractionState::default();
        let prop = world
            .spawn((Transform::from_xyz(0.0, 0.0, -2.0), RigidBody::Dynamic))
            .id();
        interaction_state.hovered_entity = Some(prop);
        world.insert_resource(interaction_state);

        let _camera = world
            .spawn((PlayerCamera, Transform::IDENTITY, GlobalTransform::IDENTITY))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(player_physics_grab);
        schedule.run(&mut world);

        assert!(world.get::<GrabbedProp>(prop).is_some());
        assert_eq!(*world.get::<RigidBody>(prop).unwrap(), RigidBody::Kinematic);

        let mut mouse_released = ButtonInput::<MouseButton>::default();
        mouse_released.press(MouseButton::Right);
        mouse_released.release(MouseButton::Right);
        world.insert_resource(mouse_released);

        schedule.run(&mut world);

        assert!(world.get::<GrabbedProp>(prop).is_none());
        assert_eq!(*world.get::<RigidBody>(prop).unwrap(), RigidBody::Dynamic);
    }

    #[test]
    fn test_player_raycast_interaction_detects_and_harvests() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            bevy::scene::ScenePlugin::default(),
            PhysicsPlugins::default(),
        ));
        app.init_asset::<Mesh>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_resource::<InteractionState>();
        app.add_systems(Update, player_raycast_interaction);

        app.world_mut().spawn((
            PlayerCamera,
            Camera::default(),
            Transform::from_xyz(0.0, 1.75, 0.0),
            GlobalTransform::from_translation(Vec3::new(0.0, 1.75, 0.0)),
        ));

        let target = app
            .world_mut()
            .spawn((
                Interactable {
                    label: "Test Ingredient",
                },
                AlchemicalIngredient {
                    id: "test_ing".to_string(),
                    name: "Test Herb".to_string(),
                    primary_effect: crate::alchemy::AlchemicalEffectKind::Vitality,
                    secondary_effect: crate::alchemy::AlchemicalEffectKind::Swiftness,
                    harvested: false,
                },
                Transform::from_xyz(0.0, 1.75, -2.0),
                GlobalTransform::from_translation(Vec3::new(0.0, 1.75, -2.0)),
            ))
            .id();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);

        app.update();

        let state = app.world().resource::<InteractionState>();
        assert_eq!(
            state.hovered_entity,
            Some(target),
            "Interactable must be detected in line of sight"
        );
    }
}
