use bevy::prelude::*;
use crate::player::{
    camera::{Health, Player, Stamina},
    interaction::InteractionState,
};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud)
            .add_systems(Update, (update_hud_bars, update_crosshair_target));
    }
}

#[derive(Component)]
pub struct CrosshairDot;

#[derive(Component)]
pub struct HealthBarFill;

#[derive(Component)]
pub struct StaminaBarFill;

fn setup_hud(mut commands: Commands) {
    // Root UI container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|root| {
            // Central Crosshair Dot
            root.spawn((
                CrosshairDot,
                Node {
                    width: Val::Px(4.0),
                    height: Val::Px(4.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
                BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            ));

            // Bottom Left Health Bar (Skyrim Red)
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(40.0),
                    bottom: Val::Px(30.0),
                    width: Val::Px(180.0),
                    height: Val::Px(8.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.05, 0.05, 0.6)),
                BorderColor(Color::srgba(0.4, 0.1, 0.1, 0.8)),
            ))
            .with_children(|bar| {
                bar.spawn((
                    HealthBarFill,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.85, 0.15, 0.15)),
                ));
            });

            // Bottom Right Stamina Bar (Skyrim Green)
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(40.0),
                    bottom: Val::Px(30.0),
                    width: Val::Px(180.0),
                    height: Val::Px(8.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.05, 0.1, 0.05, 0.6)),
                BorderColor(Color::srgba(0.1, 0.4, 0.1, 0.8)),
            ))
            .with_children(|bar| {
                bar.spawn((
                    StaminaBarFill,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.18, 0.82, 0.25)),
                ));
            });
        });
}

fn update_crosshair_target(
    interaction: Res<InteractionState>,
    mut dot_query: Query<&mut BackgroundColor, With<CrosshairDot>>,
) {
    for mut bg in dot_query.iter_mut() {
        if interaction.hovered_entity.is_some() {
            // Highlight crosshair cyan when hovering over an interactable prop
            *bg = BackgroundColor(Color::srgb(0.1, 0.95, 1.0));
        } else {
            *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.7));
        }
    }
}

fn update_hud_bars(
    player_query: Query<(&Health, &Stamina), With<Player>>,
    mut health_bar: Query<&mut Node, (With<HealthBarFill>, Without<StaminaBarFill>)>,
    mut stamina_bar: Query<&mut Node, (With<StaminaBarFill>, Without<HealthBarFill>)>,
) {
    let Ok((health, stamina)) = player_query.get_single() else {
        return;
    };

    let hp_pct = (health.current / health.max).clamp(0.0, 1.0) * 100.0;
    let st_pct = (stamina.current / stamina.max).clamp(0.0, 1.0) * 100.0;

    for mut node in health_bar.iter_mut() {
        node.width = Val::Percent(hp_pct);
    }

    for mut node in stamina_bar.iter_mut() {
        node.width = Val::Percent(st_pct);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_hud_spawns_crosshair_and_bars() {
        let mut app = App::new();
        app.add_systems(Startup, setup_hud);
        app.update();

        assert_eq!(app.world_mut().query::<&CrosshairDot>().iter(app.world()).count(), 1);
        assert_eq!(app.world_mut().query::<&HealthBarFill>().iter(app.world()).count(), 1);
        assert_eq!(app.world_mut().query::<&StaminaBarFill>().iter(app.world()).count(), 1);
    }

    #[test]
    fn test_crosshair_color_updates_on_hover() {
        let mut app = App::new();
        app.init_resource::<InteractionState>();
        app.add_systems(Update, update_crosshair_target);

        let dot = app.world_mut().spawn((
            CrosshairDot,
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
        )).id();

        // Hover over dummy entity
        let dummy = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<InteractionState>().hovered_entity = Some(dummy);

        app.update();

        let bg = app.world().get::<BackgroundColor>(dot).unwrap();
        assert_eq!(bg.0, Color::srgb(0.1, 0.95, 1.0), "Crosshair must turn cyan when hovering an interactable");

        // Clear hover
        app.world_mut().resource_mut::<InteractionState>().hovered_entity = None;
        app.update();

        let bg_cleared = app.world().get::<BackgroundColor>(dot).unwrap();
        assert_eq!(bg_cleared.0, Color::srgba(1.0, 1.0, 1.0, 0.7), "Crosshair must revert to default white");
    }

    #[test]
    fn test_hud_bars_update_with_player_stats() {
        let mut app = App::new();
        app.add_systems(Update, update_hud_bars);

        app.world_mut().spawn((
            Player,
            Health { current: 50.0, max: 100.0 }, // 50%
            Stamina { current: 25.0, max: 100.0 }, // 25%
        ));

        let health_bar = app.world_mut().spawn((
            HealthBarFill,
            Node::default(),
        )).id();

        let stamina_bar = app.world_mut().spawn((
            StaminaBarFill,
            Node::default(),
        )).id();

        app.update();

        let hp_node = app.world().get::<Node>(health_bar).unwrap();
        let st_node = app.world().get::<Node>(stamina_bar).unwrap();

        assert_eq!(hp_node.width, Val::Percent(50.0));
        assert_eq!(st_node.width, Val::Percent(25.0));
    }
}
