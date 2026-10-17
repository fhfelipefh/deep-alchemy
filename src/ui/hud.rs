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
