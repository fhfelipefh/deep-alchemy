use crate::{
    player::{
        camera::{Health, Player, Stamina},
        interaction::InteractionState,
    },
    ui::menu::GameState,
};
use bevy::prelude::*;

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
                ),
            );
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ActiveHotbarSlot(pub usize); // 1..=5

impl Default for ActiveHotbarSlot {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct CrosshairDot;

#[derive(Component)]
pub struct HealthHeartFill {
    pub index: usize, // 0..4
}

#[derive(Component)]
pub struct StaminaBarFill;

#[derive(Component)]
pub struct HotbarSlotUi {
    pub slot_index: usize, // 1..=5
}

#[derive(Component)]
pub struct PotionSlotIcon {
    pub slot_index: usize, // 1..=5
}

pub const COLOR_BORDER_GOLD: Color = Color::srgb(0.83, 0.69, 0.22); // #d4af37
pub const COLOR_BORDER_ACTIVE: Color = Color::srgb(1.0, 0.88, 0.40); // Bright Gold
pub const COLOR_BG_DARK: Color = Color::srgba(0.04, 0.06, 0.09, 0.88);
pub const COLOR_HEART_FULL: Color = Color::srgb(0.92, 0.14, 0.22);
pub const COLOR_HEART_EMPTY: Color = Color::srgba(0.20, 0.15, 0.18, 0.7);
pub const COLOR_STAMINA_FILL: Color = Color::srgb(0.15, 0.88, 0.48);

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle: Handle<Font> = asset_server.load("fonts/DejaVuSans.ttf");

    // Root UI container
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
            // -------------------------------------------------------------
            // 1. TOP-LEFT: STYLIZED HEARTS ROW & EMERALD STAMINA GAUGE
            // -------------------------------------------------------------
            root.spawn((Node {
                position_type: PositionType::Absolute,
                left: Val::Px(24.0),
                top: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },))
                .with_children(|vitality_col| {
                    // Row of Stylized Alchemical Hearts (PNG icons with black ink outline and gloss)
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
                                ));
                            }
                        });

                    // Sleek Emerald Stamina Gauge underneath
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

            // -------------------------------------------------------------
            // 2. CENTER: RETICLE / CROSSHAIR
            // -------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    width: Val::Px(6.0),
                    height: Val::Px(6.0),
                    margin: UiRect {
                        left: Val::Px(-3.0),
                        top: Val::Px(-3.0),
                        ..default()
                    },
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                CrosshairDot,
                BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
                BorderColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            ));

            // -------------------------------------------------------------
            // 3. BOTTOM-CENTER: 5-SLOT ALCHEMICAL FLASKS HOTBAR
            // -------------------------------------------------------------
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    bottom: Val::Px(22.0),
                    margin: UiRect {
                        left: Val::Px(-160.0), // Center 5x 56px + gaps
                        ..default()
                    },
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(5.0)),
                    border: UiRect::all(Val::Px(1.5)),
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
                            BackgroundColor(Color::srgba(0.06, 0.08, 0.12, 0.88)),
                            BorderColor(if slot_num == 1 {
                                COLOR_BORDER_ACTIVE
                            } else {
                                COLOR_BORDER_GOLD
                            }),
                        ))
                        .with_children(|slot| {
                            // Slot number tag (top-left aligned tag)
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

                            // Alchemical Potion Flask Icon (unique shape, color, and stopper per slot)
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

    // Calculate active heart threshold (20 HP per heart)
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

fn handle_hotbar_input(keys: Res<ButtonInput<KeyCode>>, mut active_slot: ResMut<ActiveHotbarSlot>) {
    if keys.just_pressed(KeyCode::Digit1) {
        active_slot.0 = 1;
    } else if keys.just_pressed(KeyCode::Digit2) {
        active_slot.0 = 2;
    } else if keys.just_pressed(KeyCode::Digit3) {
        active_slot.0 = 3;
    } else if keys.just_pressed(KeyCode::Digit4) {
        active_slot.0 = 4;
    } else if keys.just_pressed(KeyCode::Digit5) {
        active_slot.0 = 5;
    }
}

fn update_hotbar_selection_visuals(
    active_slot: Res<ActiveHotbarSlot>,
    mut slots_query: Query<(&HotbarSlotUi, &mut BorderColor, &mut Node)>,
) {
    if !active_slot.is_changed() {
        return;
    }

    for (slot, mut border, mut node) in slots_query.iter_mut() {
        if slot.slot_index == active_slot.0 {
            *border = BorderColor(COLOR_BORDER_ACTIVE);
            node.border = UiRect::all(Val::Px(2.5));
        } else {
            *border = BorderColor(COLOR_BORDER_GOLD);
            node.border = UiRect::all(Val::Px(1.0));
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

        // 1. Verify 5 stylized health hearts spawned
        let mut hearts_query = app.world_mut().query::<&HealthHeartFill>();
        let heart_count = hearts_query.iter(app.world()).count();
        assert_eq!(heart_count, 5, "HUD must spawn exactly 5 health heart orbs");

        // 2. Verify stamina bar fill spawned
        let mut stamina_query = app.world_mut().query::<&StaminaBarFill>();
        let stamina_count = stamina_query.iter(app.world()).count();
        assert_eq!(stamina_count, 1, "HUD must spawn stamina bar fill element");

        // 3. Verify exactly 5 hotbar slots spawned (slots 1..=5)
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

        // 4. Verify crosshair dot exists
        let mut crosshair_query = app.world_mut().query::<&CrosshairDot>();
        assert_eq!(
            crosshair_query.iter(app.world()).count(),
            1,
            "Crosshair reticle must exist"
        );

        // 5. Verify all 5 stylized alchemical potion icons spawned (slots 1..=5)
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
        app.init_resource::<ActiveHotbarSlot>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, handle_hotbar_input);

        // Press Digit3
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Digit3);
        app.update();
        assert_eq!(
            app.world().resource::<ActiveHotbarSlot>().0,
            3,
            "Pressing 3 must select slot 3"
        );

        // Clear and press Digit5
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
    fn test_hud_bars_update_with_player_stats() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()));
        app.init_asset::<Font>();
        app.init_asset::<Image>();
        app.init_resource::<ActiveHotbarSlot>();
        app.add_systems(Startup, setup_hud);
        app.add_systems(Update, update_hud_bars);
        app.update();

        // Spawn player with 60 HP (3 hearts active out of 5) and 40 Stamina
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

        // Default: not hovering
        let mut dot_query = app.world_mut().query::<(&CrosshairDot, &BackgroundColor)>();
        for (_, bg) in dot_query.iter(app.world()) {
            assert_eq!(bg.0, Color::srgba(1.0, 1.0, 1.0, 0.75));
        }

        // Set hovered entity
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

        // Initial state is MainMenu -> hidden
        let mut hud_query = app.world_mut().query::<(&HudRoot, &Visibility)>();
        for (_, vis) in hud_query.iter(app.world()) {
            assert_eq!(*vis, Visibility::Hidden, "HUD should be hidden on MainMenu");
        }

        // Switch to InGame -> visible
        *app.world_mut().resource_mut::<State<GameState>>() = State::new(GameState::InGame);
        app.update();

        let mut hud_query = app.world_mut().query::<(&HudRoot, &Visibility)>();
        for (_, vis) in hud_query.iter(app.world()) {
            assert_eq!(*vis, Visibility::Visible, "HUD must become visible InGame");
        }
    }
}
