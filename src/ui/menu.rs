use bevy::{
    app::AppExit,
    prelude::*,
    window::{CursorGrabMode, MonitorSelection, PrimaryWindow, WindowMode},
};
use crate::player::camera::PlayerCamera;

/// Global application lifecycle states.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    InGame,
}

/// Navigation subviews inside the menu screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuView {
    #[default]
    Main,
    Settings,
    Controls,
}

/// Navigation state resource tracking active view and whether the player has entered gameplay.
#[derive(Resource, Debug, Clone)]
pub struct MenuNavigation {
    pub current_view: MenuView,
    pub has_started: bool,
}

impl Default for MenuNavigation {
    fn default() -> Self {
        Self {
            current_view: MenuView::Main,
            has_started: false,
        }
    }
}

/// User configurable game settings for display, controls, and audio.
#[derive(Resource, Debug, Clone)]
pub struct GameSettings {
    pub fullscreen: bool,
    pub mouse_sensitivity: f32,
    pub fov_degrees: f32,
    pub master_volume: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            mouse_sensitivity: 0.0022,
            fov_degrees: 75.0,
            master_volume: 0.8,
        }
    }
}

// Markers for UI entities
#[derive(Component)]
pub struct MenuRoot;

#[derive(Component)]
pub struct MainViewNode;

#[derive(Component)]
pub struct SettingsViewNode;

#[derive(Component)]
pub struct ControlsViewNode;

#[derive(Component)]
pub struct StartResumeButtonText;

#[derive(Component)]
pub struct FullscreenButtonText;

#[derive(Component)]
pub struct SensitivityValueText;

#[derive(Component)]
pub struct FovValueText;

#[derive(Component)]
pub struct VolumeValueText;

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum MenuButtonAction {
    StartOrResume,
    OpenSettings,
    OpenControls,
    BackToMain,
    ToggleFullscreen,
    AdjustSensitivity(f32),
    AdjustFov(f32),
    AdjustVolume(f32),
    Quit,
}

// Visual Palette: Nordic Dark Fantasy & Alchemical Gold
pub const COLOR_BG_PANEL: Color = Color::srgba(0.06, 0.08, 0.12, 0.94);
pub const COLOR_BORDER_GOLD: Color = Color::srgba(0.80, 0.68, 0.38, 0.90);
pub const COLOR_BORDER_MUTED: Color = Color::srgba(0.45, 0.40, 0.30, 0.70);

pub const COLOR_BTN_NORMAL: Color = Color::srgba(0.12, 0.16, 0.22, 0.95);
pub const COLOR_BTN_HOVER: Color = Color::srgba(0.22, 0.35, 0.45, 1.0);
pub const COLOR_BTN_PRESSED: Color = Color::srgba(0.38, 0.28, 0.16, 1.0);

pub const COLOR_TEXT_GOLD: Color = Color::srgba(0.96, 0.84, 0.45, 1.0);
pub const COLOR_TEXT_LIGHT: Color = Color::srgba(0.92, 0.90, 0.85, 1.0);
pub const COLOR_TEXT_MUTED: Color = Color::srgba(0.70, 0.68, 0.62, 0.85);
pub const COLOR_TEXT_ACCENT: Color = Color::srgba(0.40, 0.85, 0.85, 1.0);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<MenuNavigation>()
            .init_resource::<GameSettings>()
            .add_systems(Startup, (setup_menu_ui, setup_window_icon))
            .add_systems(
                Update,
                (
                    handle_global_shortcuts,
                    handle_menu_button_interactions,
                    update_menu_subview_visibility,
                    update_settings_labels,
                    apply_camera_fov,
                    apply_window_settings,
                    apply_default_font_to_unassigned_texts,
                ),
            )
            .add_systems(OnEnter(GameState::MainMenu), on_enter_menu_state)
            .add_systems(OnEnter(GameState::InGame), on_enter_ingame_state);
    }
}

pub fn setup_window_icon(
    winit_windows: Option<NonSend<bevy::winit::WinitWindows>>,
    primary_window_query: Query<Entity, With<PrimaryWindow>>,
) {
    let Some(winit_windows) = winit_windows else {
        return;
    };
    let Ok(window_entity) = primary_window_query.get_single() else {
        return;
    };
    let Some(window) = winit_windows.get_window(window_entity) else {
        return;
    };

    if let Ok(icon_bytes) = std::fs::read("assets/icons/app_icon.png") {
        if let Ok(decoded) = image::load_from_memory(&icon_bytes) {
            let rgba = decoded.to_rgba8();
            let (width, height) = rgba.dimensions();
            if let Ok(icon) = winit::window::Icon::from_rgba(rgba.into_raw(), width, height) {
                window.set_window_icon(Some(icon));
            }
        }
    }
}

pub fn apply_default_font_to_unassigned_texts(
    asset_server: Option<Res<AssetServer>>,
    mut text_font_query: Query<&mut TextFont, Added<TextFont>>,
) {
    if let Some(server) = asset_server {
        let default_font: Handle<Font> = server.load("fonts/DejaVuSans.ttf");
        for mut text_font in text_font_query.iter_mut() {
            if text_font.font == Handle::default() {
                text_font.font = default_font.clone();
            }
        }
    }
}

pub fn setup_menu_ui(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
) {
    let font_handle: Handle<Font> = asset_server
        .as_ref()
        .map(|s| s.load("fonts/DejaVuSans.ttf"))
        .unwrap_or_default();

    // Menu Root overlay covering entire viewport
    commands
        .spawn((
            MenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.75)),
            Visibility::Inherited,
            ZIndex(100),
        ))
        .with_children(|root| {
            // Central Menu Card
            root.spawn((
                Node {
                    width: Val::Px(540.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(32.0), Val::Px(28.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    row_gap: Val::Px(16.0),
                    ..default()
                },
                BackgroundColor(COLOR_BG_PANEL),
                BorderColor(COLOR_BORDER_GOLD),
            ))
            .with_children(|card| {
                // Header Title
                card.spawn((
                    Text::new("DEEP ALCHEMY"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(COLOR_TEXT_GOLD),
                    Node::default(),
                ));

                // Subtitle
                card.spawn((
                    Text::new("EXPLORAÇÃO & ALQUIMIA EM 1ª PESSOA"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(COLOR_TEXT_MUTED),
                    Node::default(),
                ));

                // Divider Line
                card.spawn((
                    Node {
                        width: Val::Percent(90.0),
                        height: Val::Px(1.5),
                        margin: UiRect::vertical(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(COLOR_BORDER_GOLD),
                ));

                // -------------------------------------------------------------
                // Subview 1: MAIN MENU
                // -------------------------------------------------------------
                card.spawn((
                    MainViewNode,
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(12.0),
                        display: Display::Flex,
                        ..default()
                    },
                ))
                .with_children(|main_view| {
                    // Play / Resume Button
                    spawn_menu_button(
                        main_view,
                        font_handle.clone(),
                        MenuButtonAction::StartOrResume,
                        "INICIAR JORNADA",
                        Some(StartResumeButtonText),
                    );

                    // Settings Button
                    spawn_menu_button(
                        main_view,
                        font_handle.clone(),
                        MenuButtonAction::OpenSettings,
                        "CONFIGURAÇÕES",
                        None::<StartResumeButtonText>,
                    );

                    // Controls Guide Button
                    spawn_menu_button(
                        main_view,
                        font_handle.clone(),
                        MenuButtonAction::OpenControls,
                        "GUIA DE CONTROLES",
                        None::<StartResumeButtonText>,
                    );

                    // Quit Button
                    spawn_menu_button(
                        main_view,
                        font_handle.clone(),
                        MenuButtonAction::Quit,
                        "SAIR DO JOGO",
                        None::<StartResumeButtonText>,
                    );
                });

                // -------------------------------------------------------------
                // Subview 2: SETTINGS MENU
                // -------------------------------------------------------------
                card.spawn((
                    SettingsViewNode,
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(12.0),
                        display: Display::None,
                        ..default()
                    },
                ))
                .with_children(|settings_view| {
                    settings_view.spawn((
                        Text::new("CONFIGURAÇÕES DO JOGO"),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_GOLD),
                        Node::default(),
                    ));

                    // Row: Fullscreen
                    spawn_settings_row(
                        settings_view,
                        font_handle.clone(),
                        "Tela Cheia (F11):",
                        |row| {
                            row.spawn((
                                Button,
                                MenuButtonAction::ToggleFullscreen,
                                Node {
                                    width: Val::Px(240.0),
                                    height: Val::Px(36.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(COLOR_BTN_NORMAL),
                                BorderColor(COLOR_BORDER_MUTED),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    FullscreenButtonText,
                                    Text::new("DESATIVADO (JANELA)"),
                                    TextFont {
                                        font: font_handle.clone(),
                                        font_size: 13.0,
                                        ..default()
                                    },
                                    TextColor(COLOR_TEXT_LIGHT),
                                    Node::default(),
                                ));
                            });
                        },
                    );

                    // Row: Sensitivity
                    spawn_stepper_row(
                        settings_view,
                        font_handle.clone(),
                        "Sensibilidade:",
                        MenuButtonAction::AdjustSensitivity(-0.0002),
                        MenuButtonAction::AdjustSensitivity(0.0002),
                        "0.0022",
                        SensitivityValueText,
                    );

                    // Row: FOV
                    spawn_stepper_row(
                        settings_view,
                        font_handle.clone(),
                        "Campo de Visão (FOV):",
                        MenuButtonAction::AdjustFov(-5.0),
                        MenuButtonAction::AdjustFov(5.0),
                        "75°",
                        FovValueText,
                    );

                    // Row: Volume
                    spawn_stepper_row(
                        settings_view,
                        font_handle.clone(),
                        "Volume Geral:",
                        MenuButtonAction::AdjustVolume(-0.1),
                        MenuButtonAction::AdjustVolume(0.1),
                        "80%",
                        VolumeValueText,
                    );

                    // Back to Main Menu Button
                    spawn_menu_button(
                        settings_view,
                        font_handle.clone(),
                        MenuButtonAction::BackToMain,
                        "VOLTAR AO MENU",
                        None::<StartResumeButtonText>,
                    );
                });

                // -------------------------------------------------------------
                // Subview 3: CONTROLS GUIDE
                // -------------------------------------------------------------
                card.spawn((
                    ControlsViewNode,
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        display: Display::None,
                        ..default()
                    },
                ))
                .with_children(|controls_view| {
                    controls_view.spawn((
                        Text::new("COMANDOS & TECLAS DE ATALHO"),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_GOLD),
                        Node::default(),
                    ));

                    let controls_list = [
                        ("[W, A, S, D]", "Movimentação em 1ª Pessoa"),
                        ("[Shift]", "Correr (Consome Estamina)"),
                        ("[Mouse]", "Olhar / Direção da Câmera"),
                        ("[E]", "Colher Ingrediente / Interagir"),
                        ("[Botão Direito]", "Segurar Prop com Física"),
                        ("[ESC]", "Pausar / Menu & Liberar Mouse"),
                        ("[F11]", "Alternar Tela Cheia Instantaneamente"),
                    ];

                    for (key, desc) in controls_list {
                        controls_view.spawn((
                            Node {
                                width: Val::Percent(95.0),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                                ..default()
                            },
                        ))
                        .with_children(|item| {
                            item.spawn((
                                Text::new(key),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(COLOR_TEXT_ACCENT),
                                Node::default(),
                            ));
                            item.spawn((
                                Text::new(desc),
                                TextFont {
                                    font: font_handle.clone(),
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(COLOR_TEXT_LIGHT),
                                Node::default(),
                            ));
                        });
                    }

                    // Back Button
                    spawn_menu_button(
                        controls_view,
                        font_handle.clone(),
                        MenuButtonAction::BackToMain,
                        "VOLTAR AO MENU",
                        None::<StartResumeButtonText>,
                    );
                });
            });
        });
}

fn spawn_menu_button<T: Component>(
    parent: &mut ChildBuilder,
    font: Handle<Font>,
    action: MenuButtonAction,
    label: &str,
    extra_marker: Option<T>,
) {
    let mut btn = parent.spawn((
        Button,
        action,
        Node {
            width: Val::Px(340.0),
            height: Val::Px(44.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.5)),
            ..default()
        },
        BackgroundColor(COLOR_BTN_NORMAL),
        BorderColor(COLOR_BORDER_MUTED),
    ));

    btn.with_children(|b| {
        let mut txt = b.spawn((
            Text::new(label),
            TextFont {
                font,
                font_size: 15.0,
                ..default()
            },
            TextColor(COLOR_TEXT_LIGHT),
            Node::default(),
        ));
        if let Some(marker) = extra_marker {
            txt.insert(marker);
        }
    });
}

fn spawn_settings_row<F: FnOnce(&mut ChildBuilder)>(
    parent: &mut ChildBuilder,
    font: Handle<Font>,
    label: &str,
    control_builder: F,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(95.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font,
                    font_size: 13.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_LIGHT),
                Node::default(),
            ));
            control_builder(row);
        });
}

fn spawn_stepper_row<T: Component>(
    parent: &mut ChildBuilder,
    font: Handle<Font>,
    label: &str,
    dec_action: MenuButtonAction,
    inc_action: MenuButtonAction,
    initial_val: &str,
    value_marker: T,
) {
    let font_clone = font.clone();
    spawn_settings_row(parent, font.clone(), label, move |row| {
        row.spawn((
            Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
        ))
        .with_children(|controls| {
            // Decrement Button
            controls.spawn((
                Button,
                dec_action,
                Node {
                    width: Val::Px(34.0),
                    height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(COLOR_BTN_NORMAL),
                BorderColor(COLOR_BORDER_MUTED),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("-"),
                    TextFont {
                        font: font_clone.clone(),
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(COLOR_TEXT_LIGHT),
                    Node::default(),
                ));
            });

            // Value Display Box
            controls.spawn((
                Node {
                    width: Val::Px(90.0),
                    height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.05, 0.08, 0.8)),
                BorderColor(COLOR_BORDER_MUTED),
            ))
            .with_children(|val_box| {
                val_box.spawn((
                    value_marker,
                    Text::new(initial_val),
                    TextFont {
                        font: font_clone.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(COLOR_TEXT_GOLD),
                    Node::default(),
                ));
            });

            // Increment Button
            controls.spawn((
                Button,
                inc_action,
                Node {
                    width: Val::Px(34.0),
                    height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(COLOR_BTN_NORMAL),
                BorderColor(COLOR_BORDER_MUTED),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("+"),
                    TextFont {
                        font: font_clone,
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(COLOR_TEXT_LIGHT),
                    Node::default(),
                ));
            });
        });
    });
}

// -----------------------------------------------------------------------------
// Systems: Navigation, Shortcuts & Interaction
// -----------------------------------------------------------------------------

pub fn handle_global_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut nav: ResMut<MenuNavigation>,
    mut settings: ResMut<GameSettings>,
) {
    // F11: Universal Fullscreen Toggle
    if keys.just_pressed(KeyCode::F11) {
        settings.fullscreen = !settings.fullscreen;
    }

    // Escape: Pause / Resume or Subview back
    if keys.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::InGame => {
                nav.current_view = MenuView::Main;
                next_state.set(GameState::MainMenu);
            }
            GameState::MainMenu => match nav.current_view {
                MenuView::Settings | MenuView::Controls => {
                    nav.current_view = MenuView::Main;
                }
                MenuView::Main => {
                    if nav.has_started {
                        next_state.set(GameState::InGame);
                    }
                }
            },
        }
    }
}

pub fn handle_menu_button_interactions(
    mut interaction_query: Query<
        (&Interaction, &MenuButtonAction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut nav: ResMut<MenuNavigation>,
    mut settings: ResMut<GameSettings>,
    mut exit_events: EventWriter<AppExit>,
) {
    for (interaction, action, mut bg, mut border) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(COLOR_BTN_PRESSED);
                *border = BorderColor(COLOR_BORDER_GOLD);

                match action {
                    MenuButtonAction::StartOrResume => {
                        nav.has_started = true;
                        next_state.set(GameState::InGame);
                    }
                    MenuButtonAction::OpenSettings => {
                        nav.current_view = MenuView::Settings;
                    }
                    MenuButtonAction::OpenControls => {
                        nav.current_view = MenuView::Controls;
                    }
                    MenuButtonAction::BackToMain => {
                        nav.current_view = MenuView::Main;
                    }
                    MenuButtonAction::ToggleFullscreen => {
                        settings.fullscreen = !settings.fullscreen;
                    }
                    MenuButtonAction::AdjustSensitivity(delta) => {
                        settings.mouse_sensitivity =
                            (settings.mouse_sensitivity + delta).clamp(0.0006, 0.0080);
                    }
                    MenuButtonAction::AdjustFov(delta) => {
                        settings.fov_degrees = (settings.fov_degrees + delta).clamp(55.0, 115.0);
                    }
                    MenuButtonAction::AdjustVolume(delta) => {
                        settings.master_volume =
                            (settings.master_volume + delta).clamp(0.0, 1.0);
                    }
                    MenuButtonAction::Quit => {
                        exit_events.send(AppExit::Success);
                    }
                }
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(COLOR_BTN_HOVER);
                *border = BorderColor(COLOR_BORDER_GOLD);
            }
            Interaction::None => {
                *bg = BackgroundColor(COLOR_BTN_NORMAL);
                *border = BorderColor(COLOR_BORDER_MUTED);
            }
        }
    }
}

pub fn update_menu_subview_visibility(
    nav: Res<MenuNavigation>,
    mut main_views: Query<&mut Node, (With<MainViewNode>, Without<SettingsViewNode>, Without<ControlsViewNode>)>,
    mut settings_views: Query<&mut Node, (With<SettingsViewNode>, Without<MainViewNode>, Without<ControlsViewNode>)>,
    mut controls_views: Query<&mut Node, (With<ControlsViewNode>, Without<MainViewNode>, Without<SettingsViewNode>)>,
    mut start_btn_query: Query<&mut Text, With<StartResumeButtonText>>,
) {
    if !nav.is_changed() {
        return;
    }

    for mut node in main_views.iter_mut() {
        node.display = if nav.current_view == MenuView::Main {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in settings_views.iter_mut() {
        node.display = if nav.current_view == MenuView::Settings {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut node in controls_views.iter_mut() {
        node.display = if nav.current_view == MenuView::Controls {
            Display::Flex
        } else {
            Display::None
        };
    }

    // Update button text to reflect whether the game is being started or resumed
    for mut txt in start_btn_query.iter_mut() {
        txt.0 = if nav.has_started {
            "CONTINUAR EXPLORAÇÃO".to_string()
        } else {
            "INICIAR JORNADA".to_string()
        };
    }
}

pub fn update_settings_labels(
    settings: Res<GameSettings>,
    mut fullscreen_txt: Query<&mut Text, (With<FullscreenButtonText>, Without<SensitivityValueText>, Without<FovValueText>, Without<VolumeValueText>)>,
    mut sens_txt: Query<&mut Text, (With<SensitivityValueText>, Without<FullscreenButtonText>, Without<FovValueText>, Without<VolumeValueText>)>,
    mut fov_txt: Query<&mut Text, (With<FovValueText>, Without<FullscreenButtonText>, Without<SensitivityValueText>, Without<VolumeValueText>)>,
    mut vol_txt: Query<&mut Text, (With<VolumeValueText>, Without<FullscreenButtonText>, Without<SensitivityValueText>, Without<FovValueText>)>,
) {
    if !settings.is_changed() {
        return;
    }

    for mut txt in fullscreen_txt.iter_mut() {
        txt.0 = if settings.fullscreen {
            "ATIVADO (TELA CHEIA)".to_string()
        } else {
            "DESATIVADO (JANELA)".to_string()
        };
    }

    for mut txt in sens_txt.iter_mut() {
        txt.0 = format!("{:.4}", settings.mouse_sensitivity);
    }

    for mut txt in fov_txt.iter_mut() {
        txt.0 = format!("{:.0}°", settings.fov_degrees);
    }

    for mut txt in vol_txt.iter_mut() {
        txt.0 = format!("{:.0}%", (settings.master_volume * 100.0).round());
    }
}

pub fn apply_camera_fov(
    settings: Res<GameSettings>,
    mut cam_query: Query<&mut Projection, With<PlayerCamera>>,
) {
    if settings.is_changed() {
        for mut proj in cam_query.iter_mut() {
            if let Projection::Perspective(ref mut persp) = *proj {
                persp.fov = settings.fov_degrees.to_radians();
            }
        }
    }
}

pub fn apply_window_settings(
    settings: Res<GameSettings>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if settings.is_changed() {
        if let Ok(mut window) = windows.get_single_mut() {
            let target_mode = if settings.fullscreen {
                WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
            } else {
                WindowMode::Windowed
            };

            if window.mode != target_mode {
                window.mode = target_mode;
            }
        }
    }
}

pub fn on_enter_menu_state(
    mut menu_query: Query<&mut Visibility, With<MenuRoot>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    for mut vis in menu_query.iter_mut() {
        *vis = Visibility::Inherited;
    }

    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}

pub fn on_enter_ingame_state(
    mut menu_query: Query<&mut Visibility, With<MenuRoot>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    for mut vis in menu_query.iter_mut() {
        *vis = Visibility::Hidden;
    }

    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_plugin_build() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.add_plugins(MenuPlugin);
        assert!(app.is_plugin_added::<MenuPlugin>());
    }

    #[test]
    fn test_game_settings_defaults_and_clamping() {
        let mut settings = GameSettings::default();
        assert_eq!(settings.fullscreen, false);
        assert_eq!(settings.fov_degrees, 75.0);
        assert_eq!(settings.mouse_sensitivity, 0.0022);
        assert_eq!(settings.master_volume, 0.8);

        // Test clamping logic
        settings.mouse_sensitivity = (settings.mouse_sensitivity + 0.1).clamp(0.0006, 0.0080);
        assert_eq!(settings.mouse_sensitivity, 0.0080);

        settings.fov_degrees = (settings.fov_degrees - 100.0).clamp(55.0, 115.0);
        assert_eq!(settings.fov_degrees, 55.0);

        settings.master_volume = (settings.master_volume + 0.5).clamp(0.0, 1.0);
        assert_eq!(settings.master_volume, 1.0);
    }

    #[test]
    fn test_menu_navigation_defaults() {
        let nav = MenuNavigation::default();
        assert_eq!(nav.current_view, MenuView::Main);
        assert_eq!(nav.has_started, false);
    }

    #[test]
    fn test_setup_menu_ui_spawns_all_elements() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.add_systems(Startup, setup_menu_ui);
        app.update();

        assert_eq!(app.world_mut().query::<&MenuRoot>().iter(app.world()).count(), 1);
        assert_eq!(app.world_mut().query::<&MainViewNode>().iter(app.world()).count(), 1);
        assert_eq!(app.world_mut().query::<&SettingsViewNode>().iter(app.world()).count(), 1);
        assert_eq!(app.world_mut().query::<&ControlsViewNode>().iter(app.world()).count(), 1);
        assert_eq!(app.world_mut().query::<&StartResumeButtonText>().iter(app.world()).count(), 1);
        assert_eq!(app.world_mut().query::<&FullscreenButtonText>().iter(app.world()).count(), 1);
        assert_eq!(app.world_mut().query::<&SensitivityValueText>().iter(app.world()).count(), 1);
        assert_eq!(app.world_mut().query::<&FovValueText>().iter(app.world()).count(), 1);
        assert_eq!(app.world_mut().query::<&VolumeValueText>().iter(app.world()).count(), 1);
    }

    #[test]
    fn test_global_shortcuts_f11_and_escape() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<GameState>();
        app.init_resource::<MenuNavigation>();
        app.init_resource::<GameSettings>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, handle_global_shortcuts);

        // Test F11 toggles fullscreen
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::F11);
        app.update();
        assert_eq!(app.world().resource::<GameSettings>().fullscreen, true);
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();

        // Switch to InGame and test Escape pauses to MainMenu
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::InGame);
        app.update();

        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
        app.update();
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().reset_all();
        app.update();
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::MainMenu);

        // In Subview Settings, Escape returns to Main view
        app.world_mut().resource_mut::<MenuNavigation>().current_view = MenuView::Settings;
        app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(KeyCode::Escape);
        app.update();
        assert_eq!(app.world().resource::<MenuNavigation>().current_view, MenuView::Main);
    }

    #[test]
    fn test_handle_menu_button_interactions_all_actions() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<GameState>();
        app.init_resource::<MenuNavigation>();
        app.init_resource::<GameSettings>();
        app.add_event::<AppExit>();
        app.add_systems(Update, handle_menu_button_interactions);

        // Test StartOrResume
        let start_btn = app.world_mut().spawn((
            Button,
            MenuButtonAction::StartOrResume,
            Interaction::Pressed,
            BackgroundColor(COLOR_BTN_NORMAL),
            BorderColor(COLOR_BORDER_MUTED),
        )).id();
        app.update();

        assert_eq!(app.world().resource::<MenuNavigation>().has_started, true);

        // Test OpenSettings
        app.world_mut().entity_mut(start_btn).despawn();
        app.world_mut().spawn((
            Button,
            MenuButtonAction::OpenSettings,
            Interaction::Pressed,
            BackgroundColor(COLOR_BTN_NORMAL),
            BorderColor(COLOR_BORDER_MUTED),
        ));
        app.update();
        assert_eq!(app.world().resource::<MenuNavigation>().current_view, MenuView::Settings);

        // Test AdjustSensitivity, AdjustFov, AdjustVolume, ToggleFullscreen
        app.world_mut().spawn((
            Button,
            MenuButtonAction::AdjustSensitivity(0.0010),
            Interaction::Pressed,
            BackgroundColor(COLOR_BTN_NORMAL),
            BorderColor(COLOR_BORDER_MUTED),
        ));
        app.world_mut().spawn((
            Button,
            MenuButtonAction::AdjustFov(10.0),
            Interaction::Pressed,
            BackgroundColor(COLOR_BTN_NORMAL),
            BorderColor(COLOR_BORDER_MUTED),
        ));
        app.world_mut().spawn((
            Button,
            MenuButtonAction::AdjustVolume(0.1),
            Interaction::Pressed,
            BackgroundColor(COLOR_BTN_NORMAL),
            BorderColor(COLOR_BORDER_MUTED),
        ));
        app.world_mut().spawn((
            Button,
            MenuButtonAction::ToggleFullscreen,
            Interaction::Pressed,
            BackgroundColor(COLOR_BTN_NORMAL),
            BorderColor(COLOR_BORDER_MUTED),
        ));
        app.update();

        let settings = app.world().resource::<GameSettings>();
        assert_eq!(settings.fullscreen, true);
        assert!((settings.mouse_sensitivity - 0.0032).abs() < 1e-5);
        assert_eq!(settings.fov_degrees, 85.0);
        assert!((settings.master_volume - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_update_menu_subview_visibility_and_resume_label() {
        let mut app = App::new();
        app.init_resource::<MenuNavigation>();
        app.add_systems(Update, update_menu_subview_visibility);

        let main_node = app.world_mut().spawn((MainViewNode, Node { display: Display::None, ..default() })).id();
        let settings_node = app.world_mut().spawn((SettingsViewNode, Node { display: Display::None, ..default() })).id();
        let controls_node = app.world_mut().spawn((ControlsViewNode, Node { display: Display::None, ..default() })).id();
        let btn_text = app.world_mut().spawn((StartResumeButtonText, Text::new("INICIAR JORNADA"))).id();

        // When in Main view
        app.world_mut().resource_mut::<MenuNavigation>().current_view = MenuView::Main;
        app.world_mut().resource_mut::<MenuNavigation>().has_started = true;
        app.update();

        assert_eq!(app.world().get::<Node>(main_node).unwrap().display, Display::Flex);
        assert_eq!(app.world().get::<Node>(settings_node).unwrap().display, Display::None);
        assert_eq!(app.world().get::<Node>(controls_node).unwrap().display, Display::None);
        assert_eq!(app.world().get::<Text>(btn_text).unwrap().0, "CONTINUAR EXPLORAÇÃO");

        // When in Settings view
        app.world_mut().resource_mut::<MenuNavigation>().current_view = MenuView::Settings;
        app.update();
        assert_eq!(app.world().get::<Node>(main_node).unwrap().display, Display::None);
        assert_eq!(app.world().get::<Node>(settings_node).unwrap().display, Display::Flex);
    }

    #[test]
    fn test_update_settings_labels() {
        let mut app = App::new();
        app.init_resource::<GameSettings>();
        app.add_systems(Update, update_settings_labels);

        let fs_text = app.world_mut().spawn((FullscreenButtonText, Text::new(""))).id();
        let sens_text = app.world_mut().spawn((SensitivityValueText, Text::new(""))).id();
        let fov_text = app.world_mut().spawn((FovValueText, Text::new(""))).id();
        let vol_text = app.world_mut().spawn((VolumeValueText, Text::new(""))).id();

        app.world_mut().resource_mut::<GameSettings>().fullscreen = true;
        app.world_mut().resource_mut::<GameSettings>().mouse_sensitivity = 0.0035;
        app.world_mut().resource_mut::<GameSettings>().fov_degrees = 90.0;
        app.world_mut().resource_mut::<GameSettings>().master_volume = 0.65;

        app.update();

        assert_eq!(app.world().get::<Text>(fs_text).unwrap().0, "ATIVADO (TELA CHEIA)");
        assert_eq!(app.world().get::<Text>(sens_text).unwrap().0, "0.0035");
        assert_eq!(app.world().get::<Text>(fov_text).unwrap().0, "90°");
        assert_eq!(app.world().get::<Text>(vol_text).unwrap().0, "65%");
    }

    #[test]
    fn test_apply_camera_fov() {
        let mut app = App::new();
        app.init_resource::<GameSettings>();
        app.add_systems(Update, apply_camera_fov);

        let cam = app.world_mut().spawn((
            PlayerCamera,
            Projection::Perspective(PerspectiveProjection {
                fov: 75.0_f32.to_radians(),
                ..default()
            }),
        )).id();

        app.world_mut().resource_mut::<GameSettings>().fov_degrees = 90.0;
        app.update();

        if let Projection::Perspective(persp) = app.world().get::<Projection>(cam).unwrap() {
            assert!((persp.fov - 90.0_f32.to_radians()).abs() < 1e-5);
        } else {
            panic!("Expected Perspective projection");
        }
    }

    #[test]
    fn test_apply_window_settings() {
        let mut app = App::new();
        app.init_resource::<GameSettings>();
        app.add_systems(Update, apply_window_settings);

        let win = app.world_mut().spawn((
            PrimaryWindow,
            Window {
                mode: WindowMode::Windowed,
                ..default()
            },
        )).id();

        app.world_mut().resource_mut::<GameSettings>().fullscreen = true;
        app.update();

        let window = app.world().get::<Window>(win).unwrap();
        assert_eq!(window.mode, WindowMode::BorderlessFullscreen(MonitorSelection::Primary));
    }

    #[test]
    fn test_on_enter_menu_and_ingame_state() {
        let mut app = App::new();
        let menu_root = app.world_mut().spawn((MenuRoot, Visibility::Hidden)).id();
        let win = app.world_mut().spawn((
            PrimaryWindow,
            Window {
                cursor_options: bevy::window::CursorOptions {
                    grab_mode: CursorGrabMode::Locked,
                    visible: false,
                    ..default()
                },
                ..default()
            },
        )).id();

        // Run on_enter_menu_state
        app.add_systems(Update, on_enter_menu_state);
        app.update();

        assert_eq!(*app.world().get::<Visibility>(menu_root).unwrap(), Visibility::Inherited);
        let win_menu = app.world().get::<Window>(win).unwrap();
        assert_eq!(win_menu.cursor_options.grab_mode, CursorGrabMode::None);
        assert_eq!(win_menu.cursor_options.visible, true);
    }
}
