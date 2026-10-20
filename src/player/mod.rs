pub mod camera;
pub mod controller;
pub mod interaction;
pub mod ragdoll;

use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<interaction::InteractionState>()
            .init_resource::<camera::CameraShake>()
            .add_systems(Startup, camera::setup_player)
            .add_systems(
                Update,
                (
                    controller::player_movement,
                    controller::player_look,
                    controller::player_camera_effects,
                    controller::update_player_body_squash,
                    controller::update_first_person_potion_sway,
                    controller::update_held_potion_visuals,
                    interaction::player_raycast_interaction,
                    interaction::player_physics_grab,
                    ragdoll::check_player_death,
                    ragdoll::update_death_camera,
                    camera::configure_player_model_layers,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_plugin_build() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(PlayerPlugin);
        assert!(app.is_plugin_added::<PlayerPlugin>());
    }
}
