use crate::{
    player::{
        camera::{Health, Player, Stamina},
        interaction::InteractionState,
    },
    ui::menu::{GameSettings, GameState},
};
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveHotbarSlot>()
            .add_systems(Startup, setup_hud)
            .add_systems(
                Update,
                (
                    update_hud_bars,
                    update_crosshair_target,
                    update_hud_visibility,
                    handle_hotbar_input,
                    update_hotbar_selection_visuals,
                    update_hotbar_pop_animations,
                    update_critical_health_pulse,
                    update_hotbar_cooldown_visuals,
                    update_fps_counter,
                ),
            );
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ActiveHotbarSlot(pub usize);

impl Default for ActiveHotbarSlot {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct FpsCounterText;

#[derive(Component)]
pub struct CrosshairDot;

#[derive(Component)]
pub struct HealthHeartFill {
    pub index: usize,
}

#[derive(Component)]
pub struct StaminaBarFill;

#[derive(Component)]
pub struct HotbarSlotUi {
    pub slot_index: usize,
}

#[derive(Component)]
pub struct PotionSlotIcon {
    pub slot_index: usize,
}

#[derive(Component)]
pub struct HotbarContainer;

#[derive(Component)]
pub struct HotbarBottomStrip;

#[derive(Component)]
pub struct CrosshairLayer;

#[derive(Component)]
pub struct CriticalVignette;

#[derive(Component, Debug, Clone, Copy)]
pub struct HotbarPopAnimation {
    pub scale: f32,
    pub velocity: f32,
    pub flash_timer: f32,
    pub flash_color: Color,
}

impl Default for HotbarPopAnimation {
    fn default() -> Self {
        Self {
            scale: 1.0,
            velocity: 0.0,
            flash_timer: 0.0,
            flash_color: Color::NONE,
        }
    }
}

pub fn slot_potion_color(slot_index: usize) -> Color {
    match slot_index {
        1 => Color::srgb(0.2, 0.9, 0.4),
        2 => Color::srgb(0.95, 0.25, 0.2),
        3 => Color::srgb(1.0, 0.85, 0.2),
        4 => Color::srgb(0.25, 0.65, 1.0),
        5 => Color::srgb(0.92, 0.95, 1.0),
        _ => Color::srgb(0.83, 0.69, 0.22),
    }
}

pub const COLOR_BORDER_GOLD: Color = Color::srgb(0.83, 0.69, 0.22);
pub const COLOR_BORDER_ACTIVE: Color = Color::srgb(1.0, 0.88, 0.40);
pub const COLOR_BG_DARK: Color = Color::srgba(0.04, 0.06, 0.09, 0.88);
pub const COLOR_HEART_FULL: Color = Color::srgb(0.92, 0.14, 0.22);
pub const COLOR_HEART_EMPTY: Color = Color::srgba(0.20, 0.15, 0.18, 0.7);
pub const COLOR_STAMINA_FILL: Color = Color::srgb(0.15, 0.88, 0.48);

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle: Handle<Font> = asset_server.load("fonts/DejaVuSans.ttf");

    commands.spawn((
        FpsCounterText,
        Text::new(""),
        TextFont {
            font: asset_server.load("fonts/DejaVuSans-Bold.ttf"),
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.88, 0.20)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(14.0),
            right: Val::Px(18.0),
            ..default()
        },
        Visibility::Hidden,
        ZIndex(200),
    ));

    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            Visibility::Hidden,
        ))
        .with_children(|root| {

            root.spawn((Node {
                position_type: PositionType::Absolute,
                left: Val::Px(24.0),
                top: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },))
                .with_children(|vitality_col| {

                    vitality_col
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(6.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|hearts_row| {
                            for i in 0..5 {
                                hearts_row.spawn((
                                    HealthHeartFill { index: i },
                                    ImageNode::new(asset_server.load("textures/ui/heart_full.png")),
                                    Node {
                                        width: Val::Px(26.0),
                                        height: Val::Px(26.0),
                                        ..default()
                                    },
                                    Transform::default(),
                                ));
                            }
                        });

                    vitality_col
                        .spawn((
                            Node {
                                width: Val::Px(154.0),
                                height: Val::Px(7.0),
                                border: UiRect::all(Val::Px(1.2)),
                                padding: UiRect::all(Val::Px(0.5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.02, 0.08, 0.04, 0.9)),
                            BorderColor(COLOR_BORDER_GOLD),
                        ))
                        .with_children(|st_bg| {
                            st_bg.spawn((
                                StaminaBarFill,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(COLOR_STAMINA_FILL),
                            ));
                        });
                });

            root.spawn((
                CriticalVignette,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border: UiRect::all(Val::Px(14.0)),
                    ..default()
                },
                BorderColor(Color::NONE),
                BackgroundColor(Color::NONE),
                Visibility::Hidden,
                ZIndex(10),
            ));

            root.spawn((
                CrosshairLayer,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|center_layer| {
                center_layer.spawn((
                    CrosshairDot,
                    Node {
                        width: Val::Px(6.0),
                        height: Val::Px(6.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
                    BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
                ));
            });

            root.spawn((
                HotbarBottomStrip,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Px(22.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|bottom_bar| {
                bottom_bar.spawn((
                    HotbarContainer,
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        padding: UiRect::all(Val::Px(5.0)),
                        border: UiRect::all(Val::Px(1.5)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        max_width: Val::Percent(95.0),
                        ..default()
                    },
                    BackgroundColor(COLOR_BG_DARK),
                    BorderColor(COLOR_BORDER_GOLD),
                ))
                .with_children(|hotbar| {
                for slot_num in 1..=5 {
                    let potion_texture = match slot_num {
                        1 => "textures/ui/flask_green.png",
                        2 => "textures/ui/flask_red.png",
                        3 => "textures/ui/flask_yellow.png",
                        4 => "textures/ui/flask_blue.png",
                        5 => "textures/ui/flask_white.png",
                        _ => "textures/ui/flask_green.png",
                    };

                    hotbar
                        .spawn((
                            HotbarSlotUi {
                                slot_index: slot_num,
                            },
                            HotbarPopAnimation::default(),
                            Node {
                                width: Val::Px(54.0),
                                height: Val::Px(54.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(3.0)),
                                border: UiRect::all(if slot_num == 1 {
                                    Val::Px(2.5)
                                } else {
                                    Val::Px(1.0)
                                }),
                                ..default()
                            },
                            Transform::default(),
                            BackgroundColor(Color::srgba(0.06, 0.08, 0.12, 0.88)),
                            BorderColor(if slot_num == 1 {
                                COLOR_BORDER_ACTIVE
                            } else {
                                COLOR_BORDER_GOLD
                            }),
                        ))
                        .with_children(|slot| {

                            slot.spawn((
                                Text::new(format!("{slot_num}")),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(if slot_num == 1 {
                                    COLOR_BORDER_ACTIVE
                                } else {
                                    COLOR_BORDER_GOLD
                                }),
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(5.0),
                                    top: Val::Px(2.0),
                                    ..default()
                                },
                            ));

                            slot.spawn((
                                PotionSlotIcon {
                                    slot_index: slot_num,
                                },
                                ImageNode::new(asset_server.load(potion_texture)),
                                Node {
                                    width: Val::Px(38.0),
                                    height: Val::Px(38.0),
                                    margin: UiRect {
                                        top: Val::Px(4.0),
                                        ..default()
                                    },
                                    ..default()
                                },
                            ));
                        });
                }
            });
        });
    });
}

fn update_crosshair_target(
    interaction: Res<InteractionState>,
    mut dot_query: Query<&mut BackgroundColor, With<CrosshairDot>>,
) {
    for mut bg in dot_query.iter_mut() {
        if interaction.hovered_entity.is_some() {

            *bg = BackgroundColor(Color::srgb(0.1, 0.95, 1.0));
        } else {
            *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.75));
        }
    }
}

fn update_hud_bars(
    player_query: Query<(&Health, &Stamina), With<Player>>,
    mut hearts_query: Query<(&HealthHeartFill, &mut ImageNode)>,
    mut stamina_bar: Query<&mut Node, With<StaminaBarFill>>,
    asset_server: Res<AssetServer>,
) {
    let Ok((health, stamina)) = player_query.get_single() else {
        return;
    };

    let active_hearts = (health.current / 20.0).ceil() as usize;

    for (heart, mut img) in hearts_query.iter_mut() {
        if heart.index < active_hearts {
            img.image = asset_server.load("textures/ui/heart_full.png");
        } else {
            img.image = asset_server.load("textures/ui/heart_empty.png");
        }
    }

    let st_pct = (stamina.current / stamina.max).clamp(0.0, 1.0) * 100.0;
    for mut node in stamina_bar.iter_mut() {
        node.width = Val::Percent(st_pct);
    }
}

pub fn handle_hotbar_input(
    game_state: Option<Res<State<GameState>>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut active_slot: ResMut<ActiveHotbarSlot>,
    gamepads: Query<&Gamepad>,
) {
    if let Some(state) = game_state {
        if *state.get() != GameState::InGame {
            return;
        }
    }

    if let Some(ref k) = keys {
        if k.just_pressed(KeyCode::Digit1) {
            active_slot.0 = 1;
        } else if k.just_pressed(KeyCode::Digit2) {
            active_slot.0 = 2;
        } else if k.just_pressed(KeyCode::Digit3) {
            active_slot.0 = 3;
        } else if k.just_pressed(KeyCode::Digit4) {
            active_slot.0 = 4;
        } else if k.just_pressed(KeyCode::Digit5) {
            active_slot.0 = 5;
        }
    }

    let mut scroll_steps = 0_i32;
    for ev in mouse_wheel.read() {
        let delta = match ev.unit {
            MouseScrollUnit::Line => -ev.y,
            MouseScrollUnit::Pixel => {
                if ev.y.abs() >= 1.0 {
                    -ev.y.signum()
                } else {
                    0.0
                }
            }
        };
        if delta > 0.0 {
            scroll_steps += delta.round().max(1.0) as i32;
        } else if delta < 0.0 {
            scroll_steps -= (-delta).round().max(1.0) as i32;
        }
    }

    for gamepad in gamepads.iter() {
        if gamepad.just_pressed(GamepadButton::RightTrigger)
            || gamepad.just_pressed(GamepadButton::DPadRight)
        {
            scroll_steps += 1;
        }
        if gamepad.just_pressed(GamepadButton::LeftTrigger)
            || gamepad.just_pressed(GamepadButton::DPadLeft)
        {
            scroll_steps -= 1;
        }
    }

    if scroll_steps != 0 {
        let current_idx = (active_slot.0.clamp(1, 5) - 1) as i32;
        let new_idx = (current_idx + scroll_steps).rem_euclid(5);
        active_slot.0 = (new_idx + 1) as usize;
    }
}

pub fn update_hotbar_selection_visuals(
    active_slot: Res<ActiveHotbarSlot>,
    mut slots_query: Query<(&HotbarSlotUi, &mut HotbarPopAnimation, &mut BorderColor, &mut Node)>,
) {
    if !active_slot.is_changed() {
        return;
    }

    for (slot, mut anim, mut border, mut node) in slots_query.iter_mut() {
        if slot.slot_index == active_slot.0 {
            *border = BorderColor(COLOR_BORDER_ACTIVE);
            node.border = UiRect::all(Val::Px(2.5));
            anim.scale = 1.25;
            anim.velocity = 0.0;
            anim.flash_timer = 0.10;
            anim.flash_color = slot_potion_color(slot.slot_index);
        } else {
            *border = BorderColor(COLOR_BORDER_GOLD);
            node.border = UiRect::all(Val::Px(1.0));
        }
    }
}

pub fn update_hotbar_pop_animations(
    time: Res<Time>,
    active_slot: Res<ActiveHotbarSlot>,
    mut slots_query: Query<(
        &HotbarSlotUi,
        &mut HotbarPopAnimation,
        &mut Transform,
        &mut BorderColor,
        &mut Node,
    )>,
) {
    let dt = time.delta_secs();
    const SPRING_K: f32 = 120.0;
    const DAMPING_C: f32 = 18.0;

    for (slot, mut anim, mut tf, mut border, mut node) in slots_query.iter_mut() {
        let diff = 1.0 - anim.scale;
        let spring_acc = diff * SPRING_K - anim.velocity * DAMPING_C;
        let new_vel = anim.velocity + spring_acc * dt;
        anim.velocity = new_vel;
        anim.scale += new_vel * dt;

        let cur_scale = anim.scale.clamp(0.8, 1.4);
        tf.scale = Vec3::new(cur_scale, cur_scale, 1.0);

        if anim.flash_timer > 0.0 {
            anim.flash_timer = (anim.flash_timer - dt).max(0.0);
            let alpha = (anim.flash_timer / 0.10).clamp(0.0, 1.0);
            *border = BorderColor(anim.flash_color.with_alpha(alpha));
        } else if slot.slot_index == active_slot.0 {
            *border = BorderColor(COLOR_BORDER_ACTIVE);
            node.border = UiRect::all(Val::Px(2.5));
        } else {
            *border = BorderColor(COLOR_BORDER_GOLD);
            node.border = UiRect::all(Val::Px(1.0));
        }
    }
}

pub fn update_critical_health_pulse(
    time: Res<Time>,
    player_query: Query<&Health, With<Player>>,
    mut hearts_query: Query<(&HealthHeartFill, &mut Transform), (Without<CriticalVignette>, Without<Player>)>,
    mut vignette_query: Query<
        (&mut Visibility, &mut BorderColor, &mut BackgroundColor),
        (With<CriticalVignette>, Without<HealthHeartFill>, Without<Player>),
    >,
) {
    let Ok(health) = player_query.get_single() else {
        return;
    };

    let is_critical = health.current > 0.0 && health.current <= 40.0;

    if is_critical {
        let t = time.elapsed_secs();
        let beat_cycle = (t * 1.4 * std::f32::consts::TAU).sin().max(0.0).powi(3);
        let heart_scale = 1.0 + 0.28 * beat_cycle;

        let active_hearts = (health.current / 20.0).ceil() as usize;
        for (heart, mut tf) in hearts_query.iter_mut() {
            if heart.index < active_hearts {
                tf.scale = Vec3::new(heart_scale, heart_scale, 1.0);
            } else {
                tf.scale = Vec3::ONE;
            }
        }

        for (mut vis, mut border, mut bg) in vignette_query.iter_mut() {
            *vis = Visibility::Inherited;
            let alpha = 0.22 * beat_cycle;
            *border = BorderColor(Color::srgba(0.85, 0.05, 0.08, alpha * 1.4));
            *bg = BackgroundColor(Color::srgba(0.65, 0.02, 0.04, alpha * 0.35));
        }
    } else {
        for (_, mut tf) in hearts_query.iter_mut() {
            if tf.scale != Vec3::ONE {
                tf.scale = Vec3::ONE;
            }
        }
        for (mut vis, _, _) in vignette_query.iter_mut() {
            if *vis != Visibility::Hidden {
                *vis = Visibility::Hidden;
            }
        }
    }
}

pub fn update_hotbar_cooldown_visuals(
    cooldowns: Option<Res<crate::alchemy::PotionCooldowns>>,
    mut icons_query: Query<(&PotionSlotIcon, &mut ImageNode)>,
) {
    let Some(cooldowns) = cooldowns else {
        return;
    };
    for (icon, mut img) in icons_query.iter_mut() {
        let idx = icon.slot_index.clamp(1, 5);
        let timer = cooldowns.timers[idx];
        if timer > 0.0 {
            img.color = Color::srgba(0.38, 0.40, 0.44, 0.40);
        } else {
            img.color = Color::WHITE;
        }
    }
}

fn update_hud_visibility(
    game_state: Res<State<GameState>>,
    mut hud_query: Query<&mut Visibility, With<HudRoot>>,
) {
    if !game_state.is_changed() {
        return;
    }

    for mut vis in hud_query.iter_mut() {
        match game_state.get() {
            GameState::InGame => *vis = Visibility::Visible,
            GameState::MainMenu => *vis = Visibility::Hidden,
        }
    }
}

pub fn update_fps_counter(
    time: Res<Time>,
    settings: Option<Res<GameSettings>>,
    mut fps_accum: Local<f32>,
    mut timer: Local<f32>,
    mut query: Query<(&mut Text, &mut Visibility), With<FpsCounterText>>,
) {
    let show = settings.as_ref().map(|s| s.show_fps).unwrap_or(false);
    for (mut text, mut vis) in query.iter_mut() {
        if !show {
            if *vis != Visibility::Hidden {
                *vis = Visibility::Hidden;
            }
            continue;
        }

        if *vis != Visibility::Visible {
            *vis = Visibility::Visible;
        }

        let dt = time.delta_secs();
        *timer += dt;
        if dt > 0.00001 {
            let current = 1.0 / dt;
            if *fps_accum == 0.0 {
                *fps_accum = current;
                let fps_val = fps_accum.round() as u32;
                text.0 = format!("{}", fps_val);
            } else {
                *fps_accum = *fps_accum * 0.85 + current * 0.15;
            }
        }

        if *timer >= 0.10 {
            *timer = 0.0;
            let fps_val = fps_accum.round() as u32;
            text.0 = format!("{}", fps_val);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_setup_hud_spawns_hearts_and_5_slot_hotbar_gate3() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<Font>();
        app.init_asset::<Image>();
        app.init_resource::<ActiveHotbarSlot>();
        app.add_systems(Startup, setup_hud);
        app.update();

        let mut hearts_query = app.world_mut().query::<&HealthHeartFill>();
        let heart_count = hearts_query.iter(app.world()).count();
        assert_eq!(heart_count, 5, "HUD must spawn exactly 5 health heart orbs");

        let fps_count = app
            .world_mut()
            .query::<&FpsCounterText>()
            .iter(app.world())
            .count();
        assert_eq!(
            fps_count, 1,
            "HUD must spawn exactly 1 discrete FPS counter element"
        );

        let mut stamina_query = app.world_mut().query::<&StaminaBarFill>();
        let stamina_count = stamina_query.iter(app.world()).count();
        assert_eq!(stamina_count, 1, "HUD must spawn stamina bar fill element");

        let mut hotbar_query = app.world_mut().query::<&HotbarSlotUi>();
        let mut found_slots: Vec<usize> = hotbar_query
            .iter(app.world())
            .map(|s| s.slot_index)
            .collect();
        found_slots.sort();
        assert_eq!(
            found_slots,
            vec![1, 2, 3, 4, 5],
            "Hotbar must contain slots 1 through 5"
        );

        let mut crosshair_query = app.world_mut().query::<&CrosshairDot>();
        assert_eq!(
            crosshair_query.iter(app.world()).count(),
            1,
            "Crosshair reticle must exist"
        );

        let mut potion_query = app.world_mut().query::<&PotionSlotIcon>();
        let mut found_potions: Vec<usize> = potion_query
            .iter(app.world())
            .map(|p| p.slot_index)
            .collect();
        found_potions.sort();
        assert_eq!(
            found_potions,
            vec![1, 2, 3, 4, 5],
            "Hotbar must contain potion icons for slots 1 through 5"
        );
    }

    #[test]
    fn test_hotbar_number_keys_switch_active_slot() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<MouseWheel>();
        app.init_resource::<ActiveHotbarSlot>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, handle_hotbar_input);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Digit3);
        app.update();
        assert_eq!(
            app.world().resource::<ActiveHotbarSlot>().0,
            3,
            "Pressing 3 must select slot 3"
        );

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Digit5);
        app.update();
        assert_eq!(
            app.world().resource::<ActiveHotbarSlot>().0,
            5,
            "Pressing 5 must select slot 5"
        );
    }

    #[test]
    fn test_hotbar_mouse_scroll_switches_active_slot() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<MouseWheel>();
        app.init_resource::<ActiveHotbarSlot>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, handle_hotbar_input);

        let window = app.world_mut().spawn_empty().id();

        app.world_mut().send_event(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: -1.0,
            window,
        });
        app.update();
        assert_eq!(app.world().resource::<ActiveHotbarSlot>().0, 2);

        app.world_mut().send_event(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: 1.0,
            window,
        });
        app.update();
        assert_eq!(app.world().resource::<ActiveHotbarSlot>().0, 1);

        app.world_mut().send_event(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: 1.0,
            window,
        });
        app.update();
        assert_eq!(app.world().resource::<ActiveHotbarSlot>().0, 5);

        app.world_mut().send_event(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: -1.0,
            window,
        });
        app.update();
        assert_eq!(app.world().resource::<ActiveHotbarSlot>().0, 1);
    }

    #[test]
    fn test_hud_bars_update_with_player_stats() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<Font>();
        app.init_asset::<Image>();
        app.init_resource::<ActiveHotbarSlot>();
        app.add_systems(Startup, setup_hud);
        app.add_systems(Update, update_hud_bars);
        app.update();

        app.world_mut().spawn((
            Player,
            Health {
                current: 60.0,
                max: 100.0,
            },
            Stamina {
                current: 40.0,
                max: 100.0,
            },
        ));

        app.update();

        let mut stamina_query = app.world_mut().query::<(&StaminaBarFill, &Node)>();
        for (_, node) in stamina_query.iter(app.world()) {
            assert_eq!(
                node.width,
                Val::Percent(40.0),
                "Stamina bar width must reflect 40%"
            );
        }
    }

    #[test]
    fn test_crosshair_color_updates_on_hover() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<Font>();
        app.init_asset::<Image>();
        app.init_resource::<ActiveHotbarSlot>();
        app.init_resource::<InteractionState>();
        app.add_systems(Startup, setup_hud);
        app.add_systems(Update, update_crosshair_target);
        app.update();

        let mut dot_query = app.world_mut().query::<(&CrosshairDot, &BackgroundColor)>();
        for (_, bg) in dot_query.iter(app.world()) {
            assert_eq!(bg.0, Color::srgba(1.0, 1.0, 1.0, 0.75));
        }

        let dummy = app.world_mut().spawn_empty().id();
        app.world_mut()
            .resource_mut::<InteractionState>()
            .hovered_entity = Some(dummy);
        app.update();

        let mut dot_query = app.world_mut().query::<(&CrosshairDot, &BackgroundColor)>();
        for (_, bg) in dot_query.iter(app.world()) {
            assert_eq!(
                bg.0,
                Color::srgb(0.1, 0.95, 1.0),
                "Hovered crosshair must turn cyan"
            );
        }
    }

    #[test]
    fn test_update_hud_visibility_switches_with_state() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            bevy::state::app::StatesPlugin,
        ));
        app.init_asset::<Font>();
        app.init_asset::<Image>();
        app.init_resource::<ActiveHotbarSlot>();
        app.init_state::<GameState>();
        app.add_systems(Startup, setup_hud);
        app.add_systems(Update, update_hud_visibility);
        app.update();

        let mut hud_query = app.world_mut().query::<(&HudRoot, &Visibility)>();
        for (_, vis) in hud_query.iter(app.world()) {
            assert_eq!(*vis, Visibility::Hidden, "HUD should be hidden on MainMenu");
        }

        *app.world_mut().resource_mut::<State<GameState>>() = State::new(GameState::InGame);
        app.update();

        let mut hud_query = app.world_mut().query::<(&HudRoot, &Visibility)>();
        for (_, vis) in hud_query.iter(app.world()) {
            assert_eq!(*vis, Visibility::Visible, "HUD must become visible InGame");
        }
    }

    #[test]
    fn test_update_fps_counter() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<GameSettings>();
        app.add_systems(Update, update_fps_counter);

        let entity = app
            .world_mut()
            .spawn((FpsCounterText, Text::new(""), Visibility::Hidden))
            .id();

        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(entity).unwrap(),
            Visibility::Hidden
        );

        app.world_mut().resource_mut::<GameSettings>().show_fps = true;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(entity).unwrap(),
            Visibility::Visible
        );
        let text_content = &app.world().get::<Text>(entity).unwrap().0;
        assert!(!text_content.is_empty());
        assert!(text_content.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_setup_hud_spawns_centered_hotbar_container() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<Font>();
        app.init_asset::<Image>();
        app.init_resource::<ActiveHotbarSlot>();
        app.add_systems(Startup, setup_hud);
        app.update();

        let mut bottom_strip_query = app.world_mut().query::<(&HotbarBottomStrip, &Node)>();
        let mut strip_count = 0;
        for (_, node) in bottom_strip_query.iter(app.world()) {
            strip_count += 1;
            assert_eq!(node.width, Val::Percent(100.0));
            assert_eq!(node.left, Val::Px(0.0));
            assert_eq!(node.right, Val::Px(0.0));
            assert_eq!(node.justify_content, JustifyContent::Center);
            assert_eq!(node.align_items, AlignItems::Center);
        }
        assert_eq!(strip_count, 1, "Must spawn exactly 1 full-width HotbarBottomStrip");

        let mut container_query = app.world_mut().query::<(&HotbarContainer, &Node)>();
        let mut container_count = 0;
        for (_, node) in container_query.iter(app.world()) {
            container_count += 1;
            assert_eq!(node.flex_direction, FlexDirection::Row);
            assert_eq!(node.justify_content, JustifyContent::Center);
            assert_eq!(node.align_items, AlignItems::Center);
        }
        assert_eq!(container_count, 1, "Must spawn exactly 1 HotbarContainer");

        let mut crosshair_layer_query = app.world_mut().query::<(&CrosshairLayer, &Node)>();
        let mut layer_count = 0;
        for (_, node) in crosshair_layer_query.iter(app.world()) {
            layer_count += 1;
            assert_eq!(node.width, Val::Percent(100.0));
            assert_eq!(node.height, Val::Percent(100.0));
            assert_eq!(node.justify_content, JustifyContent::Center);
            assert_eq!(node.align_items, AlignItems::Center);
        }
        assert_eq!(layer_count, 1, "Must spawn exactly 1 CrosshairLayer");
    }

    #[test]
    fn test_hud_elements_centered_and_equidistant_across_all_resolutions() {
        let resolutions: &[(f32, f32, &str)] = &[
            (640.0, 480.0, "VGA 4:3"),
            (800.0, 600.0, "SVGA 4:3"),
            (1024.0, 768.0, "XGA 4:3"),
            (1280.0, 720.0, "HD 720p 16:9"),
            (1366.0, 768.0, "WXGA Laptop"),
            (1440.0, 900.0, "WXGA+ 16:10"),
            (1600.0, 900.0, "HD+ 16:9"),
            (1920.0, 1080.0, "FHD 1080p 16:9"),
            (1920.0, 1200.0, "WUXGA 16:10"),
            (2560.0, 1080.0, "UW-FHD 21:9"),
            (2560.0, 1440.0, "QHD 1440p 16:9"),
            (3440.0, 1440.0, "UWQHD 21:9"),
            (3840.0, 2160.0, "4K UHD 16:9"),
            (5120.0, 1440.0, "Dual QHD 32:9 Super Ultrawide"),
            (7680.0, 4320.0, "8K UHD 16:9"),
        ];

        let slot_size = 54.0;
        let slot_count = 5.0;
        let column_gap = 8.0;
        let padding_x = 5.0 * 2.0;
        let border_x = 1.5 * 2.0;
        let hotbar_width = slot_size * slot_count + column_gap * (slot_count - 1.0) + padding_x + border_x;
        assert_eq!(hotbar_width, 315.0);

        let hotbar_height = slot_size + 5.0 * 2.0 + 1.5 * 2.0;
        assert_eq!(hotbar_height, 67.0);

        for &(w, h, name) in resolutions {
            let left_dist = (w - hotbar_width) * 0.5;
            let right_dist = w - (left_dist + hotbar_width);

            assert!(
                (left_dist - right_dist).abs() < 0.001,
                "Hotbar must be mathematically centered with identical margins on both sides for {name} ({w}x{h}). Left: {left_dist}, Right: {right_dist}"
            );

            assert!(
                left_dist >= 0.0,
                "Hotbar start margin must be non-negative for {name} ({w}x{h}), found {left_dist}"
            );
            assert!(
                right_dist >= 0.0,
                "Hotbar end margin must be non-negative for {name} ({w}x{h}), found {right_dist}"
            );

            let bottom_offset = 22.0;
            let top_margin = h - (bottom_offset + hotbar_height);
            assert!(
                top_margin > 0.0,
                "Hotbar must not overflow vertical viewport bounds for {name} ({w}x{h})"
            );

            let crosshair_size = 6.0;
            let crosshair_left = (w - crosshair_size) * 0.5;
            let crosshair_right = w - (crosshair_left + crosshair_size);
            let crosshair_top = (h - crosshair_size) * 0.5;
            let crosshair_bottom = h - (crosshair_top + crosshair_size);

            assert!(
                (crosshair_left - crosshair_right).abs() < 0.001,
                "Crosshair must be horizontally centered for {name}"
            );
            assert!(
                (crosshair_top - crosshair_bottom).abs() < 0.001,
                "Crosshair must be vertically centered for {name}"
            );

            let vitality_width = 154.0;
            let vitality_left = 24.0;
            let vitality_right_edge = vitality_left + vitality_width;
            assert!(
                vitality_right_edge < w,
                "Vitality indicators must fit inside viewport width for {name}"
            );

            let fps_right = 18.0;
            let fps_min_x = w - fps_right - 80.0;
            assert!(
                fps_min_x > vitality_right_edge,
                "FPS counter must not overlap vitality indicators on {name}"
            );
        }
    }
}
