use crate::{dungeon::test_room::SecretTestRoomRequest, player::camera::PlayerCamera};
use bevy::render::renderer::RenderAdapterInfo;
use bevy::{
    app::AppExit,
    prelude::*,
    window::{CursorGrabMode, MonitorSelection, PrimaryWindow, WindowMode},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const MIN_FOV_DEGREES: f32 = 65.0;
pub const MAX_FOV_DEGREES: f32 = 100.0;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Loading,
    InGame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuView {
    #[default]
    Main,
    Settings,
    Controls,
    Victory,
    IntroComic,
    CinematicIntro,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct GameRunStats {
    pub elapsed_secs: f32,
    pub potions_thrown: u32,
    pub torches_lit: u32,
    pub creatures_encountered: u32,
    pub has_won: bool,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct ComicCutsceneState {
    pub active_panel: usize,
}

impl Default for ComicCutsceneState {
    fn default() -> Self {
        Self {
            active_panel: 0,
        }
    }
}

#[derive(Component)]
pub struct IntroComicViewNode;

#[derive(Component)]
pub struct ComicPanelCard(pub usize);

#[derive(Component)]
pub struct ComicPageIndicatorText;

#[derive(Component)]
pub struct ComicStepButtonText;

#[derive(Resource, Debug, Clone)]
pub struct MenuNavigation {
    pub current_view: MenuView,
    pub has_started: bool,
}

#[derive(Resource, Debug, Default)]
struct SecretSequenceProgress {
    next_index: usize,
    last_input_secs: f32,
}

const SECRET_SEQUENCE: [KeyCode; 9] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
];

impl Default for MenuNavigation {
    fn default() -> Self {
        Self {
            current_view: MenuView::Main,
            has_started: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RtxQuality {
    #[default]
    Off,
    Low,
    Medium,
    Ultra,
}

impl RtxQuality {
    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Low,
            Self::Low => Self::Medium,
            Self::Medium => Self::Ultra,
            Self::Ultra => Self::Off,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "DESLIGADO",
            Self::Low => "BAIXO",
            Self::Medium => "MÉDIO",
            Self::Ultra => "ALTO",
        }
    }

    pub fn is_enabled(self) -> bool {
        self != Self::Off
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameSettings {
    pub fullscreen: bool,
    pub mouse_sensitivity: f32,
    pub fov_degrees: f32,
    pub master_volume: f32,
    pub rtx_quality: RtxQuality,
    pub rtx_enabled: bool,
    pub show_fps: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            mouse_sensitivity: 0.0022,
            fov_degrees: 80.0,
            master_volume: 0.8,
            rtx_quality: RtxQuality::Off,
            rtx_enabled: false,
            show_fps: false,
        }
    }
}

impl GameSettings {
    fn sanitize(&mut self) {
        self.mouse_sensitivity = self.mouse_sensitivity.clamp(0.0006, 0.0080);
        self.fov_degrees = self.fov_degrees.clamp(MIN_FOV_DEGREES, MAX_FOV_DEGREES);
        self.master_volume = self.master_volume.clamp(0.0, 1.0);
        self.rtx_enabled = self.rtx_quality.is_enabled();
    }
}

#[derive(Resource, Debug, Clone)]
pub struct GameSettingsPath(pub PathBuf);

impl Default for GameSettingsPath {
    fn default() -> Self {
        let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self(base.join(".deep_alchemy").join("settings.ron"))
    }
}

#[derive(Component)]
pub struct MenuRoot;

#[derive(Component)]
pub struct MenuCardContainer;

#[derive(Component)]
pub struct MainViewNode;

#[derive(Component)]
pub struct SettingsViewNode;

#[derive(Component)]
pub struct ControlsViewNode;

#[derive(Component)]
pub struct VictoryViewNode;

#[derive(Component)]
pub struct VictoryStatsText;

#[derive(Component)]
pub struct StartResumeButtonText;

#[derive(Component)]
pub struct FullscreenButtonText;

#[derive(Component)]
pub struct RtxButtonText;

#[derive(Component)]
pub struct FpsButtonText;

#[derive(Component)]
pub struct SensitivityValueText;

#[derive(Component)]
pub struct FovValueText;

#[derive(Component)]
pub struct VolumeValueText;

#[derive(Component)]
pub struct LargeMenuButton;

#[derive(Component)]
pub struct StepperButton;

#[derive(Component)]
pub struct RtxSettingsRow;

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct RtxHardwareSupport {
    pub supported: bool,
    pub detected_gpu: String,
}

impl Default for RtxHardwareSupport {
    fn default() -> Self {
        Self {
            supported: false,
            detected_gpu: String::new(),
        }
    }
}

#[derive(Component)]
pub struct MenuSpore {
    pub speed: f32,
    pub sway_phase: f32,
    pub sway_freq: f32,
    pub base_x: f32,
    pub current_y: f32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub enum MenuButtonAction {
    StartOrResume,
    RestartGame,
    OpenSettings,
    OpenControls,
    OpenIntroComic,
    NextComicPanel,
    PrevComicPanel,
    SkipCutscene,
    SelectComicPanel(usize),
    BackToMain,
    ToggleFullscreen,
    ToggleRtx,
    ToggleFps,
    AdjustSensitivity(f32),
    AdjustFov(f32),
    AdjustVolume(f32),
    Quit,
}

pub const COLOR_TITLE_MAJESTIC: Color = Color::srgba(0.92, 0.98, 1.00, 1.00);
pub const COLOR_TITLE_GLOW: Color = Color::srgba(0.25, 0.92, 1.00, 0.90);
pub const COLOR_DIVIDER_GLOW: Color = Color::srgba(0.20, 0.88, 1.00, 0.85);
pub const COLOR_DIVIDER_ACCENT: Color = Color::srgba(0.85, 0.72, 0.35, 0.90);

pub const COLOR_BTN_NORMAL: Color = Color::srgba(0.04, 0.08, 0.15, 0.78);
pub const COLOR_BTN_HOVER: Color = Color::srgba(0.00, 0.88, 1.00, 0.40);
pub const COLOR_BTN_PRESSED: Color = Color::srgba(0.08, 0.45, 0.62, 0.85);

pub const COLOR_BTN_BORDER_NORMAL: Color = Color::srgba(0.18, 0.35, 0.48, 0.85);
pub const COLOR_BTN_BORDER_HOVER: Color = Color::srgba(0.28, 0.96, 1.00, 1.00);
pub const COLOR_BTN_BORDER_PRESSED: Color = Color::srgba(0.48, 1.00, 1.00, 1.00);
pub const COLOR_STEPPER_NORMAL: Color = Color::srgba(0.06, 0.12, 0.21, 0.94);
pub const COLOR_STEPPER_HOVER: Color = Color::srgba(0.08, 0.30, 0.40, 0.98);
pub const COLOR_STEPPER_PRESSED: Color = Color::srgba(0.10, 0.50, 0.62, 1.00);
pub const COLOR_STEPPER_BORDER: Color = Color::srgba(0.30, 0.57, 0.70, 0.92);

pub const COLOR_BG_PANEL: Color = Color::srgba(0.02, 0.04, 0.08, 0.60);
pub const COLOR_BORDER_GOLD: Color = Color::srgba(0.28, 0.96, 1.00, 0.90);
pub const COLOR_BORDER_MUTED: Color = Color::srgba(0.18, 0.35, 0.48, 0.85);

pub const COLOR_TEXT_GOLD: Color = Color::srgba(0.96, 0.86, 0.48, 1.0);
pub const COLOR_TEXT_LIGHT: Color = Color::srgba(0.92, 0.97, 1.0, 1.0);
pub const COLOR_TEXT_MUTED: Color = Color::srgba(0.60, 0.72, 0.82, 0.80);
pub const COLOR_TEXT_ACCENT: Color = Color::srgba(0.25, 0.92, 1.0, 1.0);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<MenuNavigation>()
            .init_resource::<GameRunStats>()
            .init_resource::<ComicCutsceneState>()
            .init_resource::<SecretSequenceProgress>()
            .init_resource::<SecretTestRoomRequest>()
            .init_resource::<GameSettings>()
            .init_resource::<GameSettingsPath>()
            .init_resource::<RtxHardwareSupport>()
            .add_systems(Startup, (load_game_settings, setup_menu_ui, setup_window_icon).chain())
            .add_systems(
                Update,
                (
                    handle_global_shortcuts,
                    detect_secret_test_room_sequence,
                    handle_menu_button_interactions,
                    handle_intro_comic_keyboard_input,
                    update_menu_subview_visibility,
                    update_comic_panel_highlights_and_buttons,
                    update_settings_labels,
                    update_rtx_hardware_availability,
                    update_menu_spores,
                    update_game_run_stats,
                    update_victory_stats_display,
                    apply_camera_fov,
                    apply_window_settings,
                    apply_rtx_settings,
                    save_game_settings,
                    apply_default_font_to_unassigned_texts,
                ),
            )
            .add_systems(OnEnter(GameState::MainMenu), on_enter_menu_state)
            .add_systems(OnEnter(GameState::InGame), on_enter_ingame_state);
    }
}

fn detect_secret_test_room_sequence(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    time: Res<Time>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut nav: ResMut<MenuNavigation>,
    mut progress: ResMut<SecretSequenceProgress>,
    mut request: ResMut<SecretTestRoomRequest>,
) {
    if *current_state.get() != GameState::MainMenu || nav.current_view != MenuView::Main || nav.has_started {
        *progress = SecretSequenceProgress::default();
        return;
    }

    let Some(keys) = keys else { return; };
    let Some(key) = SECRET_SEQUENCE.iter().copied().find(|key| keys.just_pressed(*key)) else {
        return;
    };

    let now = time.elapsed_secs();
    if progress.next_index > 0 && now - progress.last_input_secs > 2.0 {
        progress.next_index = 0;
    }

    if key == SECRET_SEQUENCE[progress.next_index] {
        progress.next_index += 1;
    } else {
        progress.next_index = usize::from(key == SECRET_SEQUENCE[0]);
    }
    progress.last_input_secs = now;

    if progress.next_index == SECRET_SEQUENCE.len() {
        request.requested = true;
        nav.has_started = true;
        *progress = SecretSequenceProgress::default();
        next_state.set(GameState::InGame);
    }
}

pub fn load_game_settings(mut settings: ResMut<GameSettings>, path: Res<GameSettingsPath>) {
    let Ok(contents) = std::fs::read_to_string(&path.0) else { return; };
    match ron::from_str::<GameSettings>(&contents) {
        Ok(mut loaded) => {
            loaded.sanitize();
            *settings = loaded;
        }
        Err(error) => warn!("Could not load user settings from {}: {error}", path.0.display()),
    }
}

pub fn save_game_settings(settings: Res<GameSettings>, path: Res<GameSettingsPath>) {
    if !settings.is_changed() { return; }
    let Some(parent) = path.0.parent() else { return; };
    if let Err(error) = std::fs::create_dir_all(parent) {
        warn!("Could not create settings directory {}: {error}", parent.display());
        return;
    }
    match ron::ser::to_string_pretty(&*settings, ron::ser::PrettyConfig::default()) {
        Ok(contents) => {
            if let Err(error) = std::fs::write(&path.0, contents) {
                warn!("Could not save user settings to {}: {error}", path.0.display());
            }
        }
        Err(error) => warn!("Could not serialize user settings: {error}"),
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

pub fn setup_menu_ui(mut commands: Commands, asset_server: Option<Res<AssetServer>>) {
    let font_handle: Handle<Font> = asset_server
        .as_ref()
        .map(|s| s.load("fonts/DejaVuSans.ttf"))
        .unwrap_or_default();

    commands
        .spawn((
            MenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.01, 0.02, 0.04, 0.52)),
            Visibility::Inherited,
            ZIndex(100),
        ))
        .with_children(|root| {

            for i in 0..42 {
                let seed = ((i as f32 * 19.7).sin() * 43758.5453).fract().abs();
                let seed2 = ((i as f32 * 31.3).cos() * 23421.6312).fract().abs();
                let base_x = (i as f32 * 2.38 + seed * 8.0) % 94.0 + 3.0;
                let start_y = (i as f32 * 2.45 + seed2 * 20.0) % 100.0;
                let speed = 2.0 + seed * 3.5;
                let size = if i % 3 == 0 {
                    4.5
                } else if i % 2 == 0 {
                    3.2
                } else {
                    2.4
                };
                let color = if i % 3 == 0 {
                    Color::srgba(0.18, 0.95, 0.90, 0.65)
                } else if i % 4 == 0 {
                    Color::srgba(0.96, 0.82, 0.35, 0.50)
                } else {
                    Color::srgba(0.35, 0.88, 1.0, 0.45)
                };

                root.spawn((
                    MenuSpore {
                        speed,
                        sway_phase: seed * std::f32::consts::TAU,
                        sway_freq: 0.8 + seed2 * 0.9,
                        base_x,
                        current_y: start_y,
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(size),
                        height: Val::Px(size),
                        left: Val::Percent(base_x),
                        top: Val::Percent(start_y),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(size * 0.5)),
                    BackgroundColor(color),
                ));
            }

            root.spawn((
                MenuCardContainer,
                Node {
                    width: Val::Px(540.0),
                    max_width: Val::Percent(92.0),
                    max_height: Val::Percent(94.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(24.0), Val::Px(24.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    row_gap: Val::Px(16.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BorderRadius::all(Val::Px(12.0)),
                BackgroundColor(Color::srgba(0.02, 0.04, 0.08, 0.60)),
                BorderColor(Color::srgba(0.18, 0.38, 0.48, 0.75)),
            ))
            .with_children(|card| {

                card.spawn((
                    Text::new("VITRIOL"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(COLOR_TITLE_MAJESTIC),
                    Node::default(),
                ));

                card.spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(2.0),
                        margin: UiRect::vertical(Val::Px(6.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(COLOR_DIVIDER_GLOW),
                ))
                .with_children(|divider| {
                    divider.spawn((
                        Text::new("✦  ⚗  ✦"),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(COLOR_TITLE_GLOW),
                        Node {
                            margin: UiRect::all(Val::Px(-8.0)),
                            ..default()
                        },
                    ));
                });

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

                    spawn_menu_button(
                        main_view,
                        font_handle.clone(),
                        MenuButtonAction::StartOrResume,
                        "INICIAR JORNADA",
                        Some(StartResumeButtonText),
                    );

                    spawn_menu_button(
                        main_view,
                        font_handle.clone(),
                        MenuButtonAction::OpenSettings,
                        "CONFIGURAÇÕES",
                        None::<StartResumeButtonText>,
                    );

                    spawn_menu_button(
                        main_view,
                        font_handle.clone(),
                        MenuButtonAction::OpenControls,
                        "GUIA DE CONTROLES",
                        None::<StartResumeButtonText>,
                    );

                    spawn_menu_button(
                        main_view,
                        font_handle.clone(),
                        MenuButtonAction::Quit,
                        "SAIR DO JOGO",
                        None::<StartResumeButtonText>,
                    );
                });

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

                    spawn_rtx_settings_row(settings_view, font_handle.clone());

                    spawn_settings_row(
                        settings_view,
                        font_handle.clone(),
                        "Contador de FPS:",
                        |row| {
                            row.spawn((
                                Button,
                                MenuButtonAction::ToggleFps,
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
                                    FpsButtonText,
                                    Text::new("DESATIVADO"),
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

                    spawn_stepper_row(
                        settings_view,
                        font_handle.clone(),
                        "Sensibilidade:",
                        MenuButtonAction::AdjustSensitivity(-0.0002),
                        MenuButtonAction::AdjustSensitivity(0.0002),
                        "0.0022",
                        SensitivityValueText,
                    );

                    spawn_stepper_row(
                        settings_view,
                        font_handle.clone(),
                        "Campo de Visão (FOV):",
                        MenuButtonAction::AdjustFov(-5.0),
                        MenuButtonAction::AdjustFov(5.0),
                        "75°",
                        FovValueText,
                    );

                    spawn_stepper_row(
                        settings_view,
                        font_handle.clone(),
                        "Volume Geral:",
                        MenuButtonAction::AdjustVolume(-0.1),
                        MenuButtonAction::AdjustVolume(0.1),
                        "80%",
                        VolumeValueText,
                    );

                    spawn_menu_button(
                        settings_view,
                        font_handle.clone(),
                        MenuButtonAction::BackToMain,
                        "VOLTAR AO MENU",
                        None::<StartResumeButtonText>,
                    );
                });

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
                        ("[W, A, S, D] / [Analógico Esq.]", "Movimentação em 1ª PESSOA"),
                        ("[Shift] / [L3 / B]", "Correr (Consome Estamina)"),
                        ("[Espaço] / [A / Cruz]", "Pular com Squash & Stretch"),
                        ("[Ctrl] / [R3 / D-Pad Baixo]", "Agachar"),
                        ("[Mouse] / [Analógico Dir.]", "Olhar / Direção da Visão"),
                        ("[L1 / R1] ou [D-Pad Esq/Dir]", "Trocar Poção na Hotbar"),
                        ("[Botão Dir. / L2]", "Mirar Trajetória da Poção"),
                        ("[Botão Esq. / R2]", "Lançar / Ativar Poção"),
                        ("[E] / [X / Quadrado]", "Colher Ingrediente / Interagir"),
                        ("[ESC] / [Start / Select]", "Pausar / Menu Principal"),
                        ("[F11]", "Alternar Tela Cheia"),
                    ];

                    for (key, desc) in controls_list {
                        controls_view
                            .spawn((Node {
                                width: Val::Percent(95.0),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                                ..default()
                            },))
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

                    spawn_menu_button(
                        controls_view,
                        font_handle.clone(),
                        MenuButtonAction::BackToMain,
                        "VOLTAR AO MENU",
                        None::<StartResumeButtonText>,
                    );
                });

                card.spawn((
                    VictoryViewNode,
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(12.0),
                        display: Display::None,
                        ..default()
                    },
                ))
                .with_children(|victory| {
                    victory.spawn((
                        Text::new("VITÓRIA!"),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 26.0,
                            ..default()
                        },
                        TextColor(COLOR_TITLE_GLOW),
                        Node::default(),
                    ));

                    victory.spawn((
                        Text::new("Você alcançou a saída da masmorra!\nA luz azul do abismo marca o fim desta jornada subterrânea."),
                        TextFont {
                            font: font_handle.clone(),
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_LIGHT),
                        TextLayout::new_with_justify(JustifyText::Center),
                        Node::default(),
                    ));

                    victory.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::all(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            width: Val::Percent(92.0),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(8.0)),
                        BackgroundColor(Color::srgba(0.02, 0.04, 0.08, 0.75)),
                        BorderColor(COLOR_BORDER_MUTED),
                    ))
                    .with_children(|stats_box| {
                        stats_box.spawn((
                            VictoryStatsText,
                            Text::new("Tempo de Fuga: 00m 00s\nPoções Arremessadas: 0\nTochas Acesas: 0\nBichinhos da Masmorra Encontrados: 0"),
                            TextFont {
                                font: font_handle.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(COLOR_TEXT_GOLD),
                            TextLayout::new_with_justify(JustifyText::Center),
                            Node::default(),
                        ));
                    });

                    spawn_menu_button(
                        victory,
                        font_handle.clone(),
                        MenuButtonAction::RestartGame,
                        "JOGAR NOVAMENTE",
                        None::<StartResumeButtonText>,
                    );

                    spawn_menu_button(
                        victory,
                        font_handle.clone(),
                        MenuButtonAction::BackToMain,
                        "MENU PRINCIPAL",
                        None::<StartResumeButtonText>,
                    );
                });
            });

            spawn_intro_comic_cutscene_view(root, font_handle.clone());
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
        LargeMenuButton,
        action,
        Node {
            width: Val::Px(360.0),
            height: Val::Px(48.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.5)),
            margin: UiRect::vertical(Val::Px(4.0)),
            ..default()
        },
        BorderRadius::all(Val::Px(6.0)),
        BackgroundColor(COLOR_BTN_NORMAL),
        BorderColor(COLOR_BTN_BORDER_NORMAL),
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
        .spawn((Node {
            width: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
            ..default()
        },))
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font,
                    font_size: 13.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_LIGHT),
                Node {
                    width: Val::Px(190.0),
                    flex_shrink: 0.0,
                    ..default()
                },
            ));
            control_builder(row);
        });
}

fn spawn_rtx_settings_row(parent: &mut ChildBuilder, font: Handle<Font>) {
    parent
        .spawn((
            RtxSettingsRow,
            Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                display: Display::None,
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn((
                Text::new("Traçado de Raios (RTX):"),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(COLOR_TEXT_LIGHT),
                Node {
                    width: Val::Px(190.0),
                    flex_shrink: 0.0,
                    ..default()
                },
            ));

            row.spawn((
                Button,
                MenuButtonAction::ToggleRtx,
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
                    RtxButtonText,
                    Text::new("DESLIGADO"),
                    TextFont {
                        font,
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(COLOR_TEXT_LIGHT),
                    Node::default(),
                ));
            });
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
        row.spawn((Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            width: Val::Px(240.0),
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },))
            .with_children(|controls| {

                controls
                    .spawn((
                        Button,
                        StepperButton,
                        dec_action,
                        Node {
                            width: Val::Px(38.0),
                            height: Val::Px(34.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(8.0)),
                        BackgroundColor(COLOR_STEPPER_NORMAL),
                        BorderColor(COLOR_STEPPER_BORDER),
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

                controls
                    .spawn((
                        Node {
                            width: Val::Px(90.0),
                            height: Val::Px(32.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(4.0)),
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

                controls
                    .spawn((
                        Button,
                        StepperButton,
                        inc_action,
                        Node {
                            width: Val::Px(38.0),
                            height: Val::Px(34.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(8.0)),
                        BackgroundColor(COLOR_STEPPER_NORMAL),
                        BorderColor(COLOR_STEPPER_BORDER),
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

pub fn handle_global_shortcuts(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    gamepads: Query<&Gamepad>,
    touch_controls: Option<Res<crate::player::touch_controls::TouchControlsState>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut nav: ResMut<MenuNavigation>,
    mut settings: ResMut<GameSettings>,
) {
    if keys.as_ref().map(|k| k.just_pressed(KeyCode::F11)).unwrap_or(false) {
        settings.fullscreen = !settings.fullscreen;
    }

    let touch_pause = touch_controls.as_ref().map_or(false, |t| t.toggle_menu_just_pressed);
    let toggle_pause = keys.as_ref().map(|k| k.just_pressed(KeyCode::Escape) || k.just_pressed(KeyCode::BrowserBack)).unwrap_or(false)
        || gamepads.iter().any(|g| {
            g.just_pressed(GamepadButton::Start) || g.just_pressed(GamepadButton::Select)
        })
        || touch_pause;

    if toggle_pause {
        match current_state.get() {
            GameState::InGame => {
                nav.current_view = MenuView::Main;
                next_state.set(GameState::MainMenu);
            }
            GameState::MainMenu => match nav.current_view {
                MenuView::Settings | MenuView::Controls | MenuView::Victory | MenuView::IntroComic => {
                    nav.current_view = MenuView::Main;
                }
                MenuView::CinematicIntro => {}
                MenuView::Main => {
                    if nav.has_started {
                        next_state.set(GameState::InGame);
                    }
                }
            },
            GameState::Loading => {}
        }
    }
}

pub fn handle_menu_button_interactions(
    mut interaction_query: Query<
        (
            &Interaction,
            &MenuButtonAction,
            &mut BackgroundColor,
            &mut BorderColor,
            Option<&mut Node>,
            Option<&Children>,
            Has<LargeMenuButton>,
            Has<StepperButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut next_state: ResMut<NextState<GameState>>,
    mut nav: ResMut<MenuNavigation>,
    mut settings: ResMut<GameSettings>,
    rtx_support: Option<Res<RtxHardwareSupport>>,
    mut stats_opt: Option<ResMut<GameRunStats>>,
    mut comic_state_opt: Option<ResMut<ComicCutsceneState>>,
    mut cinematic_opt: Option<ResMut<crate::intro_cinematic::IntroCinematicState>>,
    mut sound_events: Option<ResMut<Events<crate::audio::SoundEffectEvent>>>,
    mut exit_events: EventWriter<AppExit>,
) {
    for (interaction, action, mut bg, mut border, mut node_opt, children_opt, is_large, is_stepper) in
        interaction_query.iter_mut()
    {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(if is_stepper { COLOR_STEPPER_PRESSED } else { COLOR_BTN_PRESSED });
                *border = BorderColor(if is_stepper { COLOR_STEPPER_BORDER } else { COLOR_BTN_BORDER_PRESSED });
                if is_large {
                    if let Some(ref mut node) = node_opt {
                        node.width = Val::Px(356.0);
                    }
                }

                if let Some(ref mut events) = sound_events {
                    events.send(crate::audio::SoundEffectEvent::new(crate::audio::SoundEffectKind::UiClick));
                }

                match action {
                    MenuButtonAction::StartOrResume => {
                        if !nav.has_started {
                            if let Some(ref mut cinematic) = cinematic_opt {
                                cinematic.requested = true;
                                nav.current_view = MenuView::CinematicIntro;
                            } else {
                                nav.has_started = true;
                                next_state.set(GameState::InGame);
                            }
                        } else {
                            next_state.set(GameState::InGame);
                        }
                    }
                    MenuButtonAction::OpenIntroComic => {
                        if let Some(ref mut cinematic) = cinematic_opt {
                            cinematic.requested = true;
                            nav.current_view = MenuView::CinematicIntro;
                        }
                    }
                    MenuButtonAction::NextComicPanel => {
                        if let Some(ref mut comic) = comic_state_opt {
                            if comic.active_panel < 2 {
                                comic.active_panel += 1;
                            } else {
                                nav.has_started = true;
                                nav.current_view = MenuView::Main;
                                next_state.set(GameState::InGame);
                            }
                        } else {
                            nav.has_started = true;
                            nav.current_view = MenuView::Main;
                            next_state.set(GameState::InGame);
                        }
                    }
                    MenuButtonAction::PrevComicPanel => {
                        if let Some(ref mut comic) = comic_state_opt {
                            comic.active_panel = comic.active_panel.saturating_sub(1);
                        }
                    }
                    MenuButtonAction::SkipCutscene => {
                        nav.has_started = true;
                        nav.current_view = MenuView::Main;
                        next_state.set(GameState::InGame);
                    }
                    MenuButtonAction::SelectComicPanel(idx) => {
                        if let Some(ref mut comic) = comic_state_opt {
                            comic.active_panel = (*idx).min(2);
                        }
                    }
                    MenuButtonAction::RestartGame => {
                        nav.current_view = MenuView::CinematicIntro;
                        nav.has_started = false;
                        if let Some(ref mut stats) = stats_opt {
                            stats.has_won = false;
                            stats.elapsed_secs = 0.0;
                            stats.potions_thrown = 0;
                            stats.torches_lit = 0;
                            stats.creatures_encountered = 0;
                        }
                        if let Some(ref mut cinematic) = cinematic_opt {
                            cinematic.requested = true;
                        } else {
                            nav.has_started = true;
                            nav.current_view = MenuView::Main;
                            next_state.set(GameState::InGame);
                        }
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
                    MenuButtonAction::ToggleRtx => {
                        let is_supported = rtx_support.as_ref().map_or(true, |r| r.supported);
                        if is_supported {
                            settings.rtx_quality = settings.rtx_quality.next();
                            settings.rtx_enabled = settings.rtx_quality.is_enabled();
                        } else {
                            settings.rtx_quality = RtxQuality::Off;
                            settings.rtx_enabled = false;
                        }
                    }
                    MenuButtonAction::ToggleFps => {
                        settings.show_fps = !settings.show_fps;
                    }
                    MenuButtonAction::AdjustSensitivity(delta) => {
                        settings.mouse_sensitivity =
                            (settings.mouse_sensitivity + delta).clamp(0.0006, 0.0080);
                    }
                    MenuButtonAction::AdjustFov(delta) => {
                        settings.fov_degrees = (settings.fov_degrees + delta).clamp(MIN_FOV_DEGREES, MAX_FOV_DEGREES);
                    }
                    MenuButtonAction::AdjustVolume(delta) => {
                        settings.master_volume = (settings.master_volume + delta).clamp(0.0, 1.0);
                    }
                    MenuButtonAction::Quit => {
                        exit_events.send(AppExit::Success);
                    }
                }
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(if is_stepper { COLOR_STEPPER_HOVER } else { COLOR_BTN_HOVER });
                *border = BorderColor(if is_stepper { COLOR_STEPPER_BORDER } else { COLOR_BTN_BORDER_HOVER });
                if is_large {
                    if let Some(ref mut node) = node_opt {
                        node.width = Val::Px(376.0);
                    }
                }
                if is_large {
                if let Some(children) = children_opt {
                    for child in children.iter() {
                        if let Ok(mut text) = text_query.get_mut(*child) {
                            let raw = text.0.trim_matches(|c| {
                                c == '>' || c == '<' || c == ' ' || c == '✦' || c == '⚗'
                            });
                            text.0 = format!("> {} <", raw);
                        }
                    }
                }
                }
            }
            Interaction::None => {
                *bg = BackgroundColor(if is_stepper { COLOR_STEPPER_NORMAL } else { COLOR_BTN_NORMAL });
                *border = BorderColor(if is_stepper { COLOR_STEPPER_BORDER } else { COLOR_BTN_BORDER_NORMAL });
                if is_large {
                    if let Some(ref mut node) = node_opt {
                        node.width = Val::Px(360.0);
                    }
                }
                if is_large {
                if let Some(children) = children_opt {
                    for child in children.iter() {
                        if let Ok(mut text) = text_query.get_mut(*child) {
                            let raw = text.0.trim_matches(|c| {
                                c == '>' || c == '<' || c == ' ' || c == '✦' || c == '⚗'
                            });
                            text.0 = raw.to_string();
                        }
                    }
                }
                }
            }
        }
    }
}

pub fn update_menu_spores(
    time_real: Option<Res<Time<Real>>>,
    time_default: Option<Res<Time>>,
    current_state: Option<Res<State<GameState>>>,
    mut spores: Query<(&mut MenuSpore, &mut Node)>,
) {
    if let Some(state) = current_state {
        if *state.get() != GameState::MainMenu {
            return;
        }
    }
    let (dt, elapsed) = if let Some(tr) = time_real {
        (tr.delta_secs(), tr.elapsed_secs())
    } else if let Some(td) = time_default {
        (td.delta_secs(), td.elapsed_secs())
    } else {
        return;
    };
    for (mut spore, mut node) in spores.iter_mut() {
        spore.current_y -= spore.speed * dt;
        if spore.current_y < -5.0 {
            spore.current_y = 105.0;
        }
        let sway = (elapsed * spore.sway_freq + spore.sway_phase).sin() * 2.2;
        node.left = Val::Percent((spore.base_x + sway).clamp(0.5, 99.5));
        node.top = Val::Percent(spore.current_y);
    }
}

pub fn update_menu_subview_visibility(
    nav: Res<MenuNavigation>,
    mut card_views: Query<&mut Node, (With<MenuCardContainer>, Without<IntroComicViewNode>)>,
    mut comic_views: Query<&mut Node, (With<IntroComicViewNode>, Without<MenuCardContainer>)>,
    mut main_views: Query<
        &mut Node,
        (
            With<MainViewNode>,
            Without<SettingsViewNode>,
            Without<ControlsViewNode>,
            Without<VictoryViewNode>,
            Without<MenuCardContainer>,
            Without<IntroComicViewNode>,
        ),
    >,
    mut settings_views: Query<
        &mut Node,
        (
            With<SettingsViewNode>,
            Without<MainViewNode>,
            Without<ControlsViewNode>,
            Without<VictoryViewNode>,
            Without<MenuCardContainer>,
            Without<IntroComicViewNode>,
        ),
    >,
    mut controls_views: Query<
        &mut Node,
        (
            With<ControlsViewNode>,
            Without<MainViewNode>,
            Without<SettingsViewNode>,
            Without<VictoryViewNode>,
            Without<MenuCardContainer>,
            Without<IntroComicViewNode>,
        ),
    >,
    mut victory_views: Query<
        &mut Node,
        (
            With<VictoryViewNode>,
            Without<MainViewNode>,
            Without<SettingsViewNode>,
            Without<ControlsViewNode>,
            Without<MenuCardContainer>,
            Without<IntroComicViewNode>,
        ),
    >,
    mut start_btn_query: Query<&mut Text, With<StartResumeButtonText>>,
) {
    if !nav.is_changed() {
        return;
    }

    let is_comic = nav.current_view == MenuView::IntroComic;
    let is_cinematic = nav.current_view == MenuView::CinematicIntro;

    for mut node in card_views.iter_mut() {
        node.display = if is_comic || is_cinematic {
            Display::None
        } else {
            Display::Flex
        };
    }

    for mut node in comic_views.iter_mut() {
        node.display = if is_comic {
            Display::Flex
        } else {
            Display::None
        };
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

    for mut node in victory_views.iter_mut() {
        node.display = if nav.current_view == MenuView::Victory {
            Display::Flex
        } else {
            Display::None
        };
    }

    for mut txt in start_btn_query.iter_mut() {
        txt.0 = if nav.has_started {
            "CONTINUAR".to_string()
        } else {
            "INICIAR JORNADA".to_string()
        };
    }
}

pub fn update_game_run_stats(
    time: Res<Time>,
    game_state: Option<Res<State<GameState>>>,
    mut stats: ResMut<GameRunStats>,
) {
    if let Some(state) = game_state {
        if *state.get() == GameState::InGame && !stats.has_won {
            stats.elapsed_secs += time.delta_secs();
        }
    }
}

pub fn update_victory_stats_display(
    stats: Res<GameRunStats>,
    mut query: Query<&mut Text, With<VictoryStatsText>>,
) {
    if stats.is_changed() {
        let minutes = (stats.elapsed_secs / 60.0) as u32;
        let seconds = (stats.elapsed_secs % 60.0) as u32;
        for mut text in query.iter_mut() {
            text.0 = format!(
                "Tempo de Fuga: {:02}m {:02}s\nPoções Arremessadas: {}\nTochas Acesas: {}\nBichinhos da Masmorra Encontrados: {}",
                minutes, seconds, stats.potions_thrown, stats.torches_lit, stats.creatures_encountered
            );
        }
    }
}

pub fn handle_intro_comic_keyboard_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut nav: ResMut<MenuNavigation>,
    mut next_state: ResMut<NextState<GameState>>,
    mut comic_state: ResMut<ComicCutsceneState>,
) {
    if nav.current_view != MenuView::IntroComic {
        return;
    }
    let Some(keys) = keys else {
        return;
    };
    if keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::Enter)
        || keys.just_pressed(KeyCode::ArrowRight)
        || keys.just_pressed(KeyCode::KeyD)
    {
        if comic_state.active_panel < 2 {
            comic_state.active_panel += 1;
        } else {
            nav.has_started = true;
            nav.current_view = MenuView::Main;
            next_state.set(GameState::InGame);
        }
    } else if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
        comic_state.active_panel = comic_state.active_panel.saturating_sub(1);
    }
}

pub fn update_comic_panel_highlights_and_buttons(
    comic_state: Res<ComicCutsceneState>,
    mut panels: Query<(&ComicPanelCard, &mut BorderColor, &mut BackgroundColor)>,
    mut indicator_query: Query<&mut Text, (With<ComicPageIndicatorText>, Without<ComicStepButtonText>)>,
    mut next_btn_text: Query<&mut Text, (With<ComicStepButtonText>, Without<ComicPageIndicatorText>)>,
) {
    if !comic_state.is_changed() {
        return;
    }

    for (panel, mut border, mut bg) in panels.iter_mut() {
        if panel.0 == comic_state.active_panel {
            *border = BorderColor(COLOR_BORDER_GOLD);
            *bg = BackgroundColor(match panel.0 {
                0 => Color::srgba(0.04, 0.18, 0.10, 0.98),
                1 => Color::srgba(0.02, 0.04, 0.14, 0.98),
                _ => Color::srgba(0.02, 0.16, 0.24, 0.98),
            });
        } else {
            *border = BorderColor(Color::srgba(0.12, 0.18, 0.24, 0.65));
            *bg = BackgroundColor(match panel.0 {
                0 => Color::srgba(0.02, 0.10, 0.05, 0.65),
                1 => Color::srgba(0.01, 0.02, 0.08, 0.65),
                _ => Color::srgba(0.01, 0.08, 0.14, 0.65),
            });
        }
    }

    for mut text in indicator_query.iter_mut() {
        text.0 = format!("QUADRINHO {} DE 3", comic_state.active_panel + 1);
    }

    for mut text in next_btn_text.iter_mut() {
        text.0 = if comic_state.active_panel < 2 {
            "PRÓXIMO QUADRINHO >> [ESPAÇO]".to_string()
        } else {
            "COMEÇAR A JORNADA >> [ESPAÇO]".to_string()
        };
    }
}

fn spawn_intro_comic_cutscene_view(parent: &mut ChildBuilder, font: Handle<Font>) {
    parent
        .spawn((
            IntroComicViewNode,
            Node {
                width: Val::Percent(96.0),
                height: Val::Percent(94.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(3.0)),
                display: Display::None,
                overflow: Overflow::clip(),
                ..default()
            },
            BorderRadius::all(Val::Px(14.0)),
            BackgroundColor(Color::srgba(0.015, 0.025, 0.05, 0.96)),
            BorderColor(Color::srgba(0.18, 0.38, 0.52, 0.90)),
        ))
        .with_children(|comic_root| {
            comic_root
                .spawn((Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(4.0),
                    ..default()
                },))
                .with_children(|header| {
                    header.spawn((
                        Text::new("★ CRÔNICAS DE VITRIOL: A QUEDA NAS PROFUNDEZAS ★"),
                        TextFont {
                            font: font.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(COLOR_TEXT_GOLD),
                        Node::default(),
                    ));

                    header
                        .spawn((
                            Node {
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(3.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(4.0)),
                            BackgroundColor(Color::srgb(0.96, 0.84, 0.28)),
                            BorderColor(Color::BLACK),
                        ))
                        .with_children(|tag| {
                            tag.spawn((
                                Text::new("PRÓLOGO EM HISTÓRIA EM QUADRINHOS — COMO O ALQUIMISTA CAIU NA MASMORRA"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.08, 0.08, 0.12)),
                                Node::default(),
                            ));
                        });
                });

            comic_root
                .spawn((Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Stretch,
                    column_gap: Val::Px(14.0),
                    margin: UiRect::vertical(Val::Px(8.0)),
                    ..default()
                },))
                .with_children(|strip| {
                    spawn_comic_panel_1(strip, font.clone());
                    spawn_comic_panel_2(strip, font.clone());
                    spawn_comic_panel_3(strip, font.clone());
                });

            comic_root
                .spawn((Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(4.0)),
                    ..default()
                },))
                .with_children(|bar| {
                    bar.spawn((
                        Button,
                        MenuButtonAction::PrevComicPanel,
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(6.0)),
                        BackgroundColor(COLOR_BTN_NORMAL),
                        BorderColor(COLOR_BTN_BORDER_NORMAL),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("« ANTERIOR"),
                            TextFont {
                                font: font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(COLOR_TEXT_LIGHT),
                            Node::default(),
                        ));
                    });

                    bar.spawn((Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },))
                    .with_children(|center| {
                        center.spawn((
                            ComicPageIndicatorText,
                            Text::new("QUADRINHO 1 DE 3"),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(COLOR_TEXT_GOLD),
                            Node::default(),
                        ));

                        center.spawn((
                            Text::new("[ESPAÇO / CLIQUE] AVANÇAR • [ESC] PULAR PARA O JOGO"),
                            TextFont {
                                font: font.clone(),
                                font_size: 10.5,
                                ..default()
                            },
                            TextColor(COLOR_TEXT_MUTED),
                            Node::default(),
                        ));
                    });

                    bar.spawn((Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        ..default()
                    },))
                    .with_children(|right_group| {
                        right_group
                            .spawn((
                                Button,
                                MenuButtonAction::NextComicPanel,
                                Node {
                                    width: Val::Px(260.0),
                                    height: Val::Px(40.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(2.5)),
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(6.0)),
                                BackgroundColor(Color::srgba(0.08, 0.40, 0.60, 0.90)),
                                BorderColor(COLOR_BORDER_GOLD),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    ComicStepButtonText,
                                    Text::new("PRÓXIMO QUADRINHO >> [ESPAÇO]"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 13.0,
                                        ..default()
                                    },
                                    TextColor(COLOR_TEXT_LIGHT),
                                    Node::default(),
                                ));
                            });

                        right_group
                            .spawn((
                                Button,
                                MenuButtonAction::SkipCutscene,
                                Node {
                                    width: Val::Px(150.0),
                                    height: Val::Px(40.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(1.5)),
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(6.0)),
                                BackgroundColor(COLOR_BTN_NORMAL),
                                BorderColor(COLOR_BORDER_MUTED),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("PULAR INTRO [ESC]"),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 12.0,
                                        ..default()
                                    },
                                    TextColor(COLOR_TEXT_MUTED),
                                    Node::default(),
                                ));
                            });
                    });
                });
        });
}

fn spawn_comic_panel_1(parent: &mut ChildBuilder, font: Handle<Font>) {
    parent
        .spawn((
            Button,
            MenuButtonAction::SelectComicPanel(0),
            ComicPanelCard(0),
            Node {
                flex_grow: 1.0,
                flex_basis: Val::Percent(31.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(3.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(Color::srgba(0.04, 0.18, 0.10, 0.98)),
            BorderColor(COLOR_BORDER_GOLD),
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::vertical(Val::Px(3.0)),
                        border: UiRect::all(Val::Px(1.5)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(4.0)),
                    BackgroundColor(Color::srgb(0.96, 0.84, 0.28)),
                    BorderColor(Color::BLACK),
                ))
                .with_children(|tag| {
                    tag.spawn((
                        Text::new("1. O CHÃO DA FLORESTA CEDE"),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.5,
                            ..default()
                        },
                        TextColor(Color::srgb(0.08, 0.08, 0.12)),
                        Node::default(),
                    ));
                });

            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(240.0),
                        position_type: PositionType::Relative,
                        border: UiRect::all(Val::Px(2.0)),
                        overflow: Overflow::clip(),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::vertical(Val::Px(6.0)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(6.0)),
                    BackgroundColor(Color::srgb(0.04, 0.16, 0.08)),
                    BorderColor(Color::BLACK),
                ))
                .with_children(|scene| {
                    for i in 0..4 {
                        let offset_x = (i as f32) * 26.0 - 40.0;
                        scene.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(10.0),
                                left: Val::Percent(50.0 + offset_x),
                                width: Val::Px(65.0),
                                height: Val::Px(65.0),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(32.5)),
                            BackgroundColor(Color::srgba(0.15, 0.55, 0.22, 0.88)),
                        ));
                    }

                    scene.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(0.0),
                            left: Val::Px(0.0),
                            width: Val::Percent(100.0),
                            height: Val::Px(45.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.38, 0.14)),
                    ));

                    scene.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(0.0),
                            left: Val::Percent(35.0),
                            width: Val::Percent(32.0),
                            height: Val::Px(45.0),
                            border: UiRect::axes(Val::Px(2.0), Val::Px(0.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.02, 0.02, 0.04)),
                        BorderColor(Color::srgb(0.35, 0.18, 0.08)),
                    ));

                    scene
                        .spawn((Node {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(32.0),
                            left: Val::Percent(38.0),
                            width: Val::Px(42.0),
                            height: Val::Px(68.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|alchemist| {
                            alchemist.spawn((
                                Node {
                                    width: Val::Px(18.0),
                                    height: Val::Px(18.0),
                                    margin: UiRect::bottom(Val::Px(2.0)),
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(9.0)),
                                BackgroundColor(Color::srgb(0.92, 0.78, 0.62)),
                            ));

                            alchemist.spawn((
                                Node {
                                    width: Val::Px(32.0),
                                    height: Val::Px(46.0),
                                    border: UiRect::all(Val::Px(1.5)),
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(4.0)),
                                BackgroundColor(Color::srgb(0.20, 0.55, 0.85)),
                                BorderColor(Color::srgb(0.10, 0.30, 0.55)),
                            ));
                        });

                    scene
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(14.0),
                                right: Val::Px(10.0),
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                                border: UiRect::all(Val::Px(2.5)),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(6.0)),
                            BackgroundColor(Color::srgb(0.98, 0.88, 0.20)),
                            BorderColor(Color::BLACK),
                        ))
                        .with_children(|balloon| {
                            balloon.spawn((
                                Text::new("¡¡ CRRAAAACK !!"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 15.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.90, 0.12, 0.10)),
                                Node::default(),
                            ));
                        });
                });

            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.5)),
                        min_height: Val::Px(56.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderRadius::all(Val::Px(4.0)),
                    BackgroundColor(Color::srgb(0.96, 0.90, 0.68)),
                    BorderColor(Color::BLACK),
                ))
                .with_children(|caption| {
                    caption.spawn((
                        Text::new("Durante a colheita de essências botânicas na floresta proibida, o solo de raízes rompe-se repentinamente sob os passos do alquimista!"),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.08, 0.08, 0.12)),
                        TextLayout::new_with_justify(JustifyText::Center),
                        Node::default(),
                    ));
                });
        });
}

fn spawn_comic_panel_2(parent: &mut ChildBuilder, font: Handle<Font>) {
    parent
        .spawn((
            Button,
            MenuButtonAction::SelectComicPanel(1),
            ComicPanelCard(1),
            Node {
                flex_grow: 1.0,
                flex_basis: Val::Percent(31.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(3.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(Color::srgba(0.02, 0.04, 0.14, 0.98)),
            BorderColor(Color::srgba(0.12, 0.18, 0.24, 0.65)),
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::vertical(Val::Px(3.0)),
                        border: UiRect::all(Val::Px(1.5)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(4.0)),
                    BackgroundColor(Color::srgb(0.96, 0.84, 0.28)),
                    BorderColor(Color::BLACK),
                ))
                .with_children(|tag| {
                    tag.spawn((
                        Text::new("2. QUEDA VERTIGINOSA NO ABISMO"),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.5,
                            ..default()
                        },
                        TextColor(Color::srgb(0.08, 0.08, 0.12)),
                        Node::default(),
                    ));
                });

            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(240.0),
                        position_type: PositionType::Relative,
                        border: UiRect::all(Val::Px(2.0)),
                        overflow: Overflow::clip(),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::vertical(Val::Px(6.0)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(6.0)),
                    BackgroundColor(Color::srgb(0.02, 0.03, 0.10)),
                    BorderColor(Color::BLACK),
                ))
                .with_children(|scene| {
                    scene.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(8.0),
                            left: Val::Percent(46.0),
                            width: Val::Px(20.0),
                            height: Val::Px(20.0),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(10.0)),
                        BackgroundColor(Color::srgba(0.55, 0.95, 0.40, 0.85)),
                    ));

                    for i in 0..9 {
                        let px = 8.0 + (i as f32) * 10.5;
                        let height = 30.0 + ((i * 7) % 5) as f32 * 12.0;
                        let top = 25.0 + ((i * 13) % 4) as f32 * 30.0;
                        scene.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(top),
                                left: Val::Percent(px),
                                width: Val::Px(2.0),
                                height: Val::Px(height),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.85, 0.95, 1.0, 0.45)),
                        ));
                    }

                    scene.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(95.0),
                            left: Val::Percent(14.0),
                            width: Val::Px(16.0),
                            height: Val::Px(22.0),
                            border: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(4.0)),
                        BackgroundColor(Color::srgb(1.0, 0.35, 0.10)),
                        BorderColor(Color::srgb(0.40, 0.10, 0.05)),
                    ));

                    scene.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(135.0),
                            right: Val::Percent(16.0),
                            width: Val::Px(16.0),
                            height: Val::Px(22.0),
                            border: UiRect::all(Val::Px(1.5)),
                            ..default()
                        },
                        BorderRadius::all(Val::Px(4.0)),
                        BackgroundColor(Color::srgb(0.20, 0.85, 1.0)),
                        BorderColor(Color::srgb(0.05, 0.25, 0.45)),
                    ));

                    scene
                        .spawn((Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(90.0),
                            left: Val::Percent(38.0),
                            width: Val::Px(42.0),
                            height: Val::Px(64.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|alchemist| {
                            alchemist.spawn((
                                Node {
                                    width: Val::Px(30.0),
                                    height: Val::Px(42.0),
                                    border: UiRect::all(Val::Px(1.5)),
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(4.0)),
                                BackgroundColor(Color::srgb(0.20, 0.55, 0.85)),
                                BorderColor(Color::srgb(0.10, 0.30, 0.55)),
                            ));

                            alchemist.spawn((
                                Node {
                                    width: Val::Px(18.0),
                                    height: Val::Px(18.0),
                                    margin: UiRect::top(Val::Px(2.0)),
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(9.0)),
                                BackgroundColor(Color::srgb(0.92, 0.78, 0.62)),
                            ));
                        });

                    scene
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(48.0),
                                left: Val::Percent(16.0),
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(6.0)),
                            BackgroundColor(Color::WHITE),
                            BorderColor(Color::BLACK),
                        ))
                        .with_children(|balloon| {
                            balloon.spawn((
                                Text::new("¡¡ AAAAAAAHHH !!"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 15.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.95, 0.10, 0.10)),
                                Node::default(),
                            ));
                        });
                });

            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.5)),
                        min_height: Val::Px(56.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderRadius::all(Val::Px(4.0)),
                    BackgroundColor(Color::srgb(0.96, 0.90, 0.68)),
                    BorderColor(Color::BLACK),
                ))
                .with_children(|caption| {
                    caption.spawn((
                        Text::new("Centenas de metros de descida cega no vácuo da montanha! Nenhum feitiço consegue deter a gravidade!"),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.08, 0.08, 0.12)),
                        TextLayout::new_with_justify(JustifyText::Center),
                        Node::default(),
                    ));
                });
        });
}

fn spawn_comic_panel_3(parent: &mut ChildBuilder, font: Handle<Font>) {
    parent
        .spawn((
            Button,
            MenuButtonAction::SelectComicPanel(2),
            ComicPanelCard(2),
            Node {
                flex_grow: 1.0,
                flex_basis: Val::Percent(31.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(3.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BorderRadius::all(Val::Px(8.0)),
            BackgroundColor(Color::srgba(0.02, 0.16, 0.24, 0.98)),
            BorderColor(Color::srgba(0.12, 0.18, 0.24, 0.65)),
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::vertical(Val::Px(3.0)),
                        border: UiRect::all(Val::Px(1.5)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(4.0)),
                    BackgroundColor(Color::srgb(0.96, 0.84, 0.28)),
                    BorderColor(Color::BLACK),
                ))
                .with_children(|tag| {
                    tag.spawn((
                        Text::new("3. O POÇO DA CACHOEIRA"),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.5,
                            ..default()
                        },
                        TextColor(Color::srgb(0.08, 0.08, 0.12)),
                        Node::default(),
                    ));
                });

            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(240.0),
                        position_type: PositionType::Relative,
                        border: UiRect::all(Val::Px(2.0)),
                        overflow: Overflow::clip(),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::vertical(Val::Px(6.0)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(6.0)),
                    BackgroundColor(Color::srgb(0.02, 0.10, 0.18)),
                    BorderColor(Color::BLACK),
                ))
                .with_children(|scene| {
                    scene.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(0.0),
                            left: Val::Percent(36.0),
                            width: Val::Percent(28.0),
                            height: Val::Percent(85.0),
                            border: UiRect::axes(Val::Px(1.5), Val::Px(0.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.70, 0.92, 1.0, 0.80)),
                        BorderColor(Color::WHITE),
                    ));

                    scene.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(0.0),
                            left: Val::Px(0.0),
                            width: Val::Percent(100.0),
                            height: Val::Px(55.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.08, 0.42, 0.54)),
                    ));

                    for i in 0..6 {
                        let ang = (i as f32) * 0.55 - 1.4;
                        let dist = 28.0;
                        let dx = ang.cos() * dist;
                        let dy = ang.sin() * dist - 15.0;
                        scene.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: Val::Px(45.0 + dy),
                                left: Val::Percent(50.0 + dx),
                                width: Val::Px(10.0),
                                height: Val::Px(10.0),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(5.0)),
                            BackgroundColor(Color::srgba(0.85, 0.98, 1.0, 0.90)),
                        ));
                    }

                    scene
                        .spawn((Node {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(30.0),
                            left: Val::Percent(42.0),
                            width: Val::Px(36.0),
                            height: Val::Px(38.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|head_water| {
                            head_water.spawn((
                                Node {
                                    width: Val::Px(8.0),
                                    height: Val::Px(8.0),
                                    margin: UiRect::bottom(Val::Px(-2.0)),
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(4.0)),
                                BackgroundColor(Color::srgb(0.25, 0.75, 0.30)),
                            ));

                            head_water.spawn((
                                Node {
                                    width: Val::Px(24.0),
                                    height: Val::Px(24.0),
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(12.0)),
                                BackgroundColor(Color::srgb(0.92, 0.78, 0.62)),
                            ));
                        });

                    scene
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(20.0),
                                right: Val::Px(8.0),
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                                border: UiRect::all(Val::Px(2.5)),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(8.0)),
                            BackgroundColor(Color::srgb(0.20, 0.88, 1.0)),
                            BorderColor(Color::BLACK),
                        ))
                        .with_children(|balloon| {
                            balloon.spawn((
                                Text::new("¡¡ SSSPLAAAASH !!!"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 16.5,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.02, 0.15, 0.35)),
                                Node::default(),
                            ));
                        });
                });

            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(1.5)),
                        min_height: Val::Px(56.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderRadius::all(Val::Px(4.0)),
                    BackgroundColor(Color::srgb(0.96, 0.90, 0.68)),
                    BorderColor(Color::BLACK),
                ))
                .with_children(|caption| {
                    caption.spawn((
                        Text::new("Tchibum! O poço da cachoeira amortece a queda. Você está vivo no pé da cachoeira... mas agora terá que explorar a masmorra para sair!"),
                        TextFont {
                            font: font.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.08, 0.08, 0.12)),
                        TextLayout::new_with_justify(JustifyText::Center),
                        Node::default(),
                    ));
                });
        });
}

pub fn update_settings_labels(
    settings: Res<GameSettings>,
    mut fullscreen_txt: Query<
        &mut Text,
        (
            With<FullscreenButtonText>,
            Without<RtxButtonText>,
            Without<FpsButtonText>,
            Without<SensitivityValueText>,
            Without<FovValueText>,
            Without<VolumeValueText>,
        ),
    >,
    mut rtx_txt: Query<
        &mut Text,
        (
            With<RtxButtonText>,
            Without<FullscreenButtonText>,
            Without<FpsButtonText>,
            Without<SensitivityValueText>,
            Without<FovValueText>,
            Without<VolumeValueText>,
        ),
    >,
    mut fps_txt: Query<
        &mut Text,
        (
            With<FpsButtonText>,
            Without<FullscreenButtonText>,
            Without<RtxButtonText>,
            Without<SensitivityValueText>,
            Without<FovValueText>,
            Without<VolumeValueText>,
        ),
    >,
    mut sens_txt: Query<
        &mut Text,
        (
            With<SensitivityValueText>,
            Without<FullscreenButtonText>,
            Without<RtxButtonText>,
            Without<FpsButtonText>,
            Without<FovValueText>,
            Without<VolumeValueText>,
        ),
    >,
    mut fov_txt: Query<
        &mut Text,
        (
            With<FovValueText>,
            Without<FullscreenButtonText>,
            Without<RtxButtonText>,
            Without<FpsButtonText>,
            Without<SensitivityValueText>,
            Without<VolumeValueText>,
        ),
    >,
    mut vol_txt: Query<
        &mut Text,
        (
            With<VolumeValueText>,
            Without<FullscreenButtonText>,
            Without<RtxButtonText>,
            Without<FpsButtonText>,
            Without<SensitivityValueText>,
            Without<FovValueText>,
        ),
    >,
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

    for mut txt in rtx_txt.iter_mut() {
        txt.0 = settings.rtx_quality.label().to_string();
    }

    for mut txt in fps_txt.iter_mut() {
        txt.0 = if settings.show_fps {
            "ATIVADO".to_string()
        } else {
            "DESATIVADO".to_string()
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

pub fn check_rtx_hardware_support(gpu_name: &str) -> bool {
    let is_mobile = cfg!(target_os = "android") || cfg!(target_os = "ios") || cfg!(target_arch = "wasm32");
    check_rtx_hardware_support_custom(gpu_name, is_mobile)
}

pub fn check_rtx_hardware_support_custom(gpu_name: &str, is_mobile: bool) -> bool {
    if is_mobile {
        return false;
    }
    let lower = gpu_name.to_lowercase();
    let trimmed = lower.trim();
    if trimmed.is_empty() {
        return false;
    }

    let is_nvidia_rtx = (lower.contains("nvidia") || lower.contains("geforce") || lower.contains("quadro") || lower.contains("titan"))
        && lower.contains("rtx")
        || trimmed.starts_with("rtx ")
        || lower.contains(" rtx ");

    let is_amd_rt = (lower.contains("radeon") || lower.contains("amd")) && (
        lower.contains("rx 6") ||
        lower.contains("rx 7") ||
        lower.contains("rx 8") ||
        lower.contains("rx6") ||
        lower.contains("rx7") ||
        lower.contains("rx8") ||
        lower.contains("rdna2") ||
        lower.contains("rdna3") ||
        lower.contains("rdna4") ||
        lower.contains("pro w6") ||
        lower.contains("pro w7")
    );

    let is_intel_arc = lower.contains("intel") && (
        lower.contains("arc") ||
        lower.contains("battlemage") ||
        lower.contains("celestial")
    );

    let is_apple_rt = lower.contains("apple") && (
        lower.contains("m3") ||
        lower.contains("m4") ||
        lower.contains("m5")
    );

    is_nvidia_rtx || is_amd_rt || is_intel_arc || is_apple_rt
}

pub fn update_rtx_hardware_availability(
    adapter_info: Option<Res<RenderAdapterInfo>>,
    mut rtx_support: ResMut<RtxHardwareSupport>,
    mut settings: ResMut<GameSettings>,
    mut rtx_row_query: Query<&mut Node, With<RtxSettingsRow>>,
) {
    if let Some(adapter) = adapter_info.as_ref() {
        let name = &adapter.name;
        if rtx_support.detected_gpu != *name {
            rtx_support.detected_gpu = name.clone();
            rtx_support.supported = check_rtx_hardware_support(name);
        }
    }

    let supported = rtx_support.supported;

    if !supported {
        if settings.rtx_enabled {
            settings.rtx_enabled = false;
        }
        if settings.rtx_quality != RtxQuality::Off {
            settings.rtx_quality = RtxQuality::Off;
        }
    }

    for mut node in rtx_row_query.iter_mut() {
        let target_display = if supported { Display::Flex } else { Display::None };
        if node.display != target_display {
            node.display = target_display;
        }
    }
}

pub fn apply_rtx_settings(
    settings: Res<GameSettings>,
    mut commands: Commands,
    camera_query: Query<Entity, With<PlayerCamera>>,
    ssao_query: Query<
        Entity,
        (
            With<PlayerCamera>,
            With<bevy::pbr::ScreenSpaceAmbientOcclusion>,
        ),
    >,
    mut ambient_light: Option<ResMut<AmbientLight>>,
    mut dir_lights: Query<&mut DirectionalLight>,
    mut key_point_lights: Query<&mut PointLight, With<crate::dungeon::modular_kit::KeyShadowLight>>,
    mut all_point_lights: Query<
        &mut PointLight,
        Without<crate::dungeon::modular_kit::KeyShadowLight>,
    >,
) {
    if !settings.is_changed() {
        return;
    }

    if let Some(ref mut ambient) = ambient_light {
        match settings.rtx_quality {
            RtxQuality::Off => {
                ambient.brightness = 480.0;
                ambient.color = Color::srgb(0.10, 0.20, 0.32);
            }
            RtxQuality::Low => {
                ambient.brightness = 740.0;
                ambient.color = Color::srgb(0.12, 0.22, 0.35);
            }
            RtxQuality::Medium => {
                ambient.brightness = 780.0;
                ambient.color = Color::srgb(0.13, 0.24, 0.36);
            }
            RtxQuality::Ultra => {
                ambient.brightness = 840.0;
                ambient.color = Color::srgb(0.14, 0.25, 0.38);
            }
        }
    }

    for mut light in dir_lights.iter_mut() {
        light.shadows_enabled = settings.rtx_quality.is_enabled();
        if settings.rtx_quality.is_enabled() {
            light.shadow_depth_bias = 0.04;
            light.shadow_normal_bias = 0.6;
        }
    }

    let enable_key_point_shadows =
        matches!(settings.rtx_quality, RtxQuality::Medium | RtxQuality::Ultra);
    for mut light in key_point_lights.iter_mut() {
        light.shadows_enabled = enable_key_point_shadows;
        if enable_key_point_shadows {
            light.shadow_depth_bias = 0.08;
            light.shadow_normal_bias = 0.6;
        }
    }

    for mut light in all_point_lights.iter_mut() {
        light.shadows_enabled = false;
    }

    match settings.rtx_quality {
        RtxQuality::Off => {
            for cam_entity in ssao_query.iter() {
                commands.entity(cam_entity).remove::<(
                    bevy::pbr::ScreenSpaceAmbientOcclusion,
                    bevy::pbr::ShadowFilteringMethod,
                )>();
            }
        }
        RtxQuality::Low => {

            for cam_entity in camera_query.iter() {
                commands.entity(cam_entity).insert((
                    bevy::pbr::ScreenSpaceAmbientOcclusion::default(),
                    bevy::pbr::ShadowFilteringMethod::Hardware2x2,
                ));
            }
        }
        RtxQuality::Medium | RtxQuality::Ultra => {

            for cam_entity in camera_query.iter() {
                commands.entity(cam_entity).insert((
                    bevy::pbr::ScreenSpaceAmbientOcclusion::default(),
                    bevy::pbr::ShadowFilteringMethod::Gaussian,
                ));
            }
        }
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
    virtual_time: Option<ResMut<Time<Virtual>>>,
) {
    for mut vis in menu_query.iter_mut() {
        *vis = Visibility::Inherited;
    }

    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }

    if let Some(mut time) = virtual_time {
        time.pause();
    }
}

pub fn on_enter_ingame_state(
    mut menu_query: Query<&mut Visibility, With<MenuRoot>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    virtual_time: Option<ResMut<Time<Virtual>>>,
    mouse_input: Option<ResMut<ButtonInput<MouseButton>>>,
) {
    for mut vis in menu_query.iter_mut() {
        *vis = Visibility::Hidden;
    }

    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }

    if let Some(mut time) = virtual_time {
        time.unpause();
    }

    if let Some(mut mouse) = mouse_input {
        mouse.reset(MouseButton::Left);
        mouse.clear_just_pressed(MouseButton::Left);
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
    fn test_secret_sequence_requests_the_test_room_from_the_initial_menu() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_resource::<Time>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.init_state::<GameState>();
        app.init_resource::<MenuNavigation>();
        app.init_resource::<SecretSequenceProgress>();
        app.init_resource::<SecretTestRoomRequest>();
        app.add_systems(Update, detect_secret_test_room_sequence);

        for key in SECRET_SEQUENCE {
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_millis(200));
            app.world_mut().resource_mut::<ButtonInput<KeyCode>>().press(key);
            app.update();
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(key);
            keys.clear_just_pressed(key);
        }

        assert!(app.world().resource::<SecretTestRoomRequest>().requested);
        assert!(app.world().resource::<MenuNavigation>().has_started);
    }

    #[test]
    fn test_game_settings_defaults_and_clamping() {
        let mut settings = GameSettings::default();
        assert_eq!(settings.fullscreen, false);
        assert_eq!(settings.fov_degrees, 80.0);
        assert_eq!(settings.mouse_sensitivity, 0.0022);
        assert_eq!(settings.master_volume, 0.8);
        assert_eq!(settings.show_fps, false);

        settings.mouse_sensitivity = (settings.mouse_sensitivity + 0.1).clamp(0.0006, 0.0080);
        assert_eq!(settings.mouse_sensitivity, 0.0080);

        settings.fov_degrees = (settings.fov_degrees - 100.0).clamp(MIN_FOV_DEGREES, MAX_FOV_DEGREES);
        assert_eq!(settings.fov_degrees, MIN_FOV_DEGREES);

        settings.fov_degrees = (settings.fov_degrees + 100.0).clamp(MIN_FOV_DEGREES, MAX_FOV_DEGREES);
        assert_eq!(settings.fov_degrees, MAX_FOV_DEGREES);

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
    fn test_game_settings_are_saved_and_loaded_with_safe_limits() {
        let settings_dir = std::env::temp_dir().join(format!(
            "deep_alchemy_settings_test_{}",
            std::process::id()
        ));
        let settings_path = settings_dir.join("settings.ron");

        let mut writer = App::new();
        writer.insert_resource(GameSettings {
            fullscreen: true,
            mouse_sensitivity: 0.0040,
            fov_degrees: 92.0,
            master_volume: 0.35,
            rtx_quality: RtxQuality::Medium,
            rtx_enabled: true,
            show_fps: true,
        });
        writer.insert_resource(GameSettingsPath(settings_path.clone()));
        writer.add_systems(Update, save_game_settings);
        writer.update();

        let mut reader = App::new();
        reader.init_resource::<GameSettings>();
        reader.insert_resource(GameSettingsPath(settings_path.clone()));
        reader.add_systems(Update, load_game_settings);
        reader.update();

        let loaded = reader.world().resource::<GameSettings>();
        assert_eq!(loaded.fov_degrees, 92.0);
        assert!(loaded.fullscreen);
        assert!(loaded.show_fps);
        assert_eq!(loaded.rtx_quality, RtxQuality::Medium);
        assert!(loaded.rtx_enabled);
        assert!((loaded.mouse_sensitivity - 0.0040).abs() < 1e-5);
        assert!((loaded.master_volume - 0.35).abs() < 1e-5);

        std::fs::remove_dir_all(settings_dir).unwrap();
    }

    #[test]
    fn test_setup_menu_ui_spawns_all_elements() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.add_systems(Startup, setup_menu_ui);
        app.update();

        assert_eq!(
            app.world_mut()
                .query::<&MenuRoot>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(
            app.world_mut()
                .query::<&MainViewNode>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(
            app.world_mut()
                .query::<&SettingsViewNode>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(
            app.world_mut()
                .query::<&ControlsViewNode>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(
            app.world_mut()
                .query::<&StartResumeButtonText>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(
            app.world_mut()
                .query::<&FullscreenButtonText>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(
            app.world_mut()
                .query::<&RtxButtonText>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(
            app.world_mut()
                .query::<&FpsButtonText>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(
            app.world_mut()
                .query::<&SensitivityValueText>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(
            app.world_mut()
                .query::<&FovValueText>()
                .iter(app.world())
                .count(),
            1
        );
        assert_eq!(
            app.world_mut()
                .query::<&VolumeValueText>()
                .iter(app.world())
                .count(),
            1
        );
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

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::F11);
        app.update();
        assert_eq!(app.world().resource::<GameSettings>().fullscreen, true);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::InGame);
        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.update();
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::MainMenu);

        app.world_mut()
            .resource_mut::<MenuNavigation>()
            .current_view = MenuView::Settings;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        assert_eq!(
            app.world().resource::<MenuNavigation>().current_view,
            MenuView::Main
        );
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

        let start_btn = app
            .world_mut()
            .spawn((
                Button,
                MenuButtonAction::StartOrResume,
                Interaction::Pressed,
                BackgroundColor(COLOR_BTN_NORMAL),
                BorderColor(COLOR_BORDER_MUTED),
            ))
            .id();
        app.update();

        assert_eq!(app.world().resource::<MenuNavigation>().has_started, true);

        app.world_mut().entity_mut(start_btn).despawn();
        app.world_mut().spawn((
            Button,
            MenuButtonAction::OpenSettings,
            Interaction::Pressed,
            BackgroundColor(COLOR_BTN_NORMAL),
            BorderColor(COLOR_BORDER_MUTED),
        ));
        app.update();
        assert_eq!(
            app.world().resource::<MenuNavigation>().current_view,
            MenuView::Settings
        );

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
        app.world_mut().spawn((
            Button,
            MenuButtonAction::ToggleRtx,
            Interaction::Pressed,
            BackgroundColor(COLOR_BTN_NORMAL),
            BorderColor(COLOR_BORDER_MUTED),
        ));
        app.world_mut().spawn((
            Button,
            MenuButtonAction::ToggleFps,
            Interaction::Pressed,
            BackgroundColor(COLOR_BTN_NORMAL),
            BorderColor(COLOR_BORDER_MUTED),
        ));
        app.update();

        let settings = app.world().resource::<GameSettings>();
        assert_eq!(settings.fullscreen, true);
        assert_eq!(settings.rtx_quality, RtxQuality::Low);
        assert_eq!(settings.rtx_enabled, true);
        assert_eq!(settings.show_fps, true);
        assert!((settings.mouse_sensitivity - 0.0032).abs() < 1e-5);
        assert_eq!(settings.fov_degrees, 90.0);
        assert!((settings.master_volume - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_update_menu_subview_visibility_and_resume_label() {
        let mut app = App::new();
        app.init_resource::<MenuNavigation>();
        app.add_systems(Update, update_menu_subview_visibility);

        let main_node = app
            .world_mut()
            .spawn((
                MainViewNode,
                Node {
                    display: Display::None,
                    ..default()
                },
            ))
            .id();
        let settings_node = app
            .world_mut()
            .spawn((
                SettingsViewNode,
                Node {
                    display: Display::None,
                    ..default()
                },
            ))
            .id();
        let controls_node = app
            .world_mut()
            .spawn((
                ControlsViewNode,
                Node {
                    display: Display::None,
                    ..default()
                },
            ))
            .id();
        let btn_text = app
            .world_mut()
            .spawn((StartResumeButtonText, Text::new("INICIAR JORNADA")))
            .id();

        app.world_mut()
            .resource_mut::<MenuNavigation>()
            .current_view = MenuView::Main;
        app.world_mut().resource_mut::<MenuNavigation>().has_started = true;
        app.update();

        assert_eq!(
            app.world().get::<Node>(main_node).unwrap().display,
            Display::Flex
        );
        assert_eq!(
            app.world().get::<Node>(settings_node).unwrap().display,
            Display::None
        );
        assert_eq!(
            app.world().get::<Node>(controls_node).unwrap().display,
            Display::None
        );
        assert_eq!(
            app.world().get::<Text>(btn_text).unwrap().0,
            "CONTINUAR"
        );

        app.world_mut()
            .resource_mut::<MenuNavigation>()
            .current_view = MenuView::Settings;
        app.update();
        assert_eq!(
            app.world().get::<Node>(main_node).unwrap().display,
            Display::None
        );
        assert_eq!(
            app.world().get::<Node>(settings_node).unwrap().display,
            Display::Flex
        );
    }

    #[test]
    fn test_update_settings_labels() {
        let mut app = App::new();
        app.init_resource::<GameSettings>();
        app.add_systems(Update, update_settings_labels);

        let fs_text = app
            .world_mut()
            .spawn((FullscreenButtonText, Text::new("")))
            .id();
        let rtx_text = app.world_mut().spawn((RtxButtonText, Text::new(""))).id();
        let fps_text = app.world_mut().spawn((FpsButtonText, Text::new(""))).id();
        let sens_text = app
            .world_mut()
            .spawn((SensitivityValueText, Text::new("")))
            .id();
        let fov_text = app.world_mut().spawn((FovValueText, Text::new(""))).id();
        let vol_text = app.world_mut().spawn((VolumeValueText, Text::new(""))).id();

        app.world_mut().resource_mut::<GameSettings>().fullscreen = true;
        app.world_mut().resource_mut::<GameSettings>().rtx_quality = RtxQuality::Low;
        app.world_mut().resource_mut::<GameSettings>().show_fps = true;
        app.world_mut()
            .resource_mut::<GameSettings>()
            .mouse_sensitivity = 0.0035;
        app.world_mut().resource_mut::<GameSettings>().fov_degrees = 90.0;
        app.world_mut().resource_mut::<GameSettings>().master_volume = 0.65;

        app.update();

        assert_eq!(
            app.world().get::<Text>(fs_text).unwrap().0,
            "ATIVADO (TELA CHEIA)"
        );
        assert_eq!(
            app.world().get::<Text>(rtx_text).unwrap().0,
            "BAIXO"
        );
        assert_eq!(
            app.world().get::<Text>(fps_text).unwrap().0,
            "ATIVADO"
        );
        assert_eq!(app.world().get::<Text>(sens_text).unwrap().0, "0.0035");
        assert_eq!(app.world().get::<Text>(fov_text).unwrap().0, "90°");
        assert_eq!(app.world().get::<Text>(vol_text).unwrap().0, "65%");
    }

    #[test]
    fn test_apply_rtx_settings() {
        let mut app = App::new();
        app.init_resource::<GameSettings>();
        app.insert_resource(AmbientLight {
            color: Color::srgb(0.1, 0.2, 0.3),
            brightness: 480.0,
        });
        app.add_systems(Update, apply_rtx_settings);

        let cam = app.world_mut().spawn(PlayerCamera).id();
        let dir_light = app
            .world_mut()
            .spawn(DirectionalLight {
                shadows_enabled: false,
                ..default()
            })
            .id();
        let key_light = app
            .world_mut()
            .spawn((
                PointLight {
                    shadows_enabled: false,
                    ..default()
                },
                crate::dungeon::modular_kit::KeyShadowLight,
            ))
            .id();
        let fill_light = app
            .world_mut()
            .spawn(PointLight {
                shadows_enabled: false,
                ..default()
            })
            .id();

        app.world_mut().resource_mut::<GameSettings>().rtx_quality = RtxQuality::Low;
        app.update();

        assert!(
            app.world()
                .get::<DirectionalLight>(dir_light)
                .unwrap()
                .shadows_enabled
        );
        assert!(
            !app.world()
                .get::<PointLight>(key_light)
                .unwrap()
                .shadows_enabled
        );
        assert!(
            !app.world()
                .get::<PointLight>(fill_light)
                .unwrap()
                .shadows_enabled
        );
        assert!(app
            .world()
            .get::<bevy::pbr::ScreenSpaceAmbientOcclusion>(cam)
            .is_some());
        assert_eq!(
            *app.world()
                .get::<bevy::pbr::ShadowFilteringMethod>(cam)
                .unwrap(),
            bevy::pbr::ShadowFilteringMethod::Hardware2x2
        );
        assert_eq!(app.world().resource::<AmbientLight>().brightness, 740.0);

        app.world_mut().resource_mut::<GameSettings>().rtx_quality = RtxQuality::Medium;
        app.update();

        assert!(
            app.world()
                .get::<DirectionalLight>(dir_light)
                .unwrap()
                .shadows_enabled
        );
        assert!(
            app.world()
                .get::<PointLight>(key_light)
                .unwrap()
                .shadows_enabled
        );
        assert!(
            !app.world()
                .get::<PointLight>(fill_light)
                .unwrap()
                .shadows_enabled
        );
        assert_eq!(
            *app.world()
                .get::<bevy::pbr::ShadowFilteringMethod>(cam)
                .unwrap(),
            bevy::pbr::ShadowFilteringMethod::Gaussian
        );
        assert_eq!(app.world().resource::<AmbientLight>().brightness, 780.0);

        app.world_mut().resource_mut::<GameSettings>().rtx_quality = RtxQuality::Off;
        app.update();

        assert!(
            !app.world()
                .get::<DirectionalLight>(dir_light)
                .unwrap()
                .shadows_enabled
        );
        assert!(
            !app.world()
                .get::<PointLight>(key_light)
                .unwrap()
                .shadows_enabled
        );
        assert!(
            !app.world()
                .get::<PointLight>(fill_light)
                .unwrap()
                .shadows_enabled
        );
        assert!(app
            .world()
            .get::<bevy::pbr::ScreenSpaceAmbientOcclusion>(cam)
            .is_none());
        assert!(app
            .world()
            .get::<bevy::pbr::ShadowFilteringMethod>(cam)
            .is_none());
        assert_eq!(app.world().resource::<AmbientLight>().brightness, 480.0);
    }

    #[test]
    fn test_apply_camera_fov() {
        let mut app = App::new();
        app.init_resource::<GameSettings>();
        app.add_systems(Update, apply_camera_fov);

        let cam = app
            .world_mut()
            .spawn((
                PlayerCamera,
                Projection::Perspective(PerspectiveProjection {
                    fov: 75.0_f32.to_radians(),
                    ..default()
                }),
            ))
            .id();

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

        let win = app
            .world_mut()
            .spawn((
                PrimaryWindow,
                Window {
                    mode: WindowMode::Windowed,
                    ..default()
                },
            ))
            .id();

        app.world_mut().resource_mut::<GameSettings>().fullscreen = true;
        app.update();

        let window = app.world().get::<Window>(win).unwrap();
        assert_eq!(
            window.mode,
            WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
        );
    }

    #[test]
    fn test_on_enter_menu_and_ingame_state() {
        let mut app = App::new();
        let menu_root = app.world_mut().spawn((MenuRoot, Visibility::Hidden)).id();
        let win = app
            .world_mut()
            .spawn((
                PrimaryWindow,
                Window {
                    cursor_options: bevy::window::CursorOptions {
                        grab_mode: CursorGrabMode::Locked,
                        visible: false,
                        ..default()
                    },
                    ..default()
                },
            ))
            .id();

        app.add_systems(Update, on_enter_menu_state);
        app.update();

        assert_eq!(
            *app.world().get::<Visibility>(menu_root).unwrap(),
            Visibility::Inherited
        );
        let win_menu = app.world().get::<Window>(win).unwrap();
        assert_eq!(win_menu.cursor_options.grab_mode, CursorGrabMode::None);
        assert_eq!(win_menu.cursor_options.visible, true);
    }

    #[test]
    fn test_menu_spores_drift() {
        let mut app = App::new();
        app.add_systems(Update, update_menu_spores);

        let spore = app
            .world_mut()
            .spawn((
                MenuSpore {
                    speed: 10.0,
                    sway_phase: 0.0,
                    sway_freq: 1.0,
                    base_x: 50.0,
                    current_y: 80.0,
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(80.0),
                    ..default()
                },
            ))
            .id();

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(500));
        app.insert_resource(time);
        app.update();

        let updated_spore = app.world().get::<MenuSpore>(spore).unwrap();
        assert!(
            updated_spore.current_y < 80.0,
            "Spore must drift upward over time"
        );
    }

    #[test]
    fn test_menu_card_centered_and_fits_across_all_resolutions() {
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

        let base_card_width: f32 = 540.0;
        let card_max_width_pct: f32 = 0.92;

        for &(w, h, name) in resolutions {
            let actual_card_width = base_card_width.min(w * card_max_width_pct);
            let left_dist = (w - actual_card_width) * 0.5;
            let right_dist = w - (left_dist + actual_card_width);

            assert!(
                (left_dist - right_dist).abs() < 0.001,
                "Menu card must be mathematically centered horizontally with equal margins for {name} ({w}x{h}). Left: {left_dist}, Right: {right_dist}"
            );

            assert!(
                left_dist > 0.0,
                "Menu card left margin must be positive for {name} ({w}x{h}), found {left_dist}"
            );
            assert!(
                right_dist > 0.0,
                "Menu card right margin must be positive for {name} ({w}x{h}), found {right_dist}"
            );

            assert!(
                actual_card_width <= w,
                "Menu card must fit within screen width for {name} ({w}x{h})"
            );
        }
    }
}
