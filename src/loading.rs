use crate::ui::menu::GameState;
use bevy::{asset::{LoadState, UntypedHandle}, prelude::*};

const PRELOAD_ASSETS: &[&str] = &[
    "fonts/DejaVuSans.ttf",
    "fonts/DejaVuSans-Bold.ttf",
    "textures/ui/heart_full.png",
    "textures/ui/heart_empty.png",
    "models/player.glb#Scene0",
    "models/intro_cinematic.glb#Scene0",
    "models/cavern_kit.glb#Scene0",
    "models/alchemy_props.glb#Scene0",
    "models/faceted_rocks_kit.glb#Scene0",
    "models/dungeon_kit.glb#Scene0",
    "models/env_cave_wall.glb#Scene0",
    "models/prop_mushroom_cyan.glb#Scene0",
    "models/prop_wooden_bench.glb#Scene0",
    "models/prop_faceted_rock.glb#Scene0",
    "models/tile_corridor_straight.glb#Scene0",
    "models/tile_corridor_corner.glb#Scene0",
    "models/tile_outer_corner.glb#Scene0",
    "models/tile_straight_edge.glb#Scene0",
    "models/tile_corridor_open.glb#Scene0",
    "models/tile_solid_pillar.glb#Scene0",
    "models/tile_grotto_connector.glb#Scene0",
    "models/macro_alchemical_shrine.glb#Scene0",
    "models/macro_crystal_cluster.glb#Scene0",
    "models/macro_crystal_cluster_b.glb#Scene0",
    "models/macro_crystal_cluster_c.glb#Scene0",
    "models/creature_sproutling.glb#Scene0",
    "models/creature_emberling.glb#Scene0",
    "models/creature_glaceling.glb#Scene0",
    "audio/potion_throw.wav",
    "audio/potion_shatter.wav",
    "audio/water_splash.wav",
    "audio/torch_ignite.wav",
    "audio/ice_freeze.wav",
    "audio/creature_chirp.wav",
    "audio/ui_click.wav",
    "audio/water_drop.wav",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadingTarget {
    #[default]
    Menu,
    InGame,
}

#[derive(Resource, Debug, Default)]
pub struct LoadingRequest {
    pub target: LoadingTarget,
}

#[derive(Resource, Debug, Default)]
pub struct WorldBootstrap {
    pub requested: bool,
    pub built: bool,
}

#[derive(Resource)]
pub struct GameplayPreload {
    pub handles: Vec<UntypedHandle>,
}

#[derive(Resource, Default)]
pub struct LoadingProgress {
    pub ready_frames: u8,
}

#[derive(Component)]
pub struct LoadingOverlay;

#[derive(Component)]
pub struct LoadingStatusText;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingRequest>()
            .init_resource::<LoadingProgress>()
            .add_systems(Startup, (begin_preload, setup_loading_overlay).chain())
            .add_systems(OnEnter(GameState::Loading), show_loading_overlay)
            .add_systems(
                Update,
                update_loading_progress.run_if(in_state(GameState::Loading)),
            )
            .add_systems(OnExit(GameState::Loading), hide_loading_overlay);
    }
}

pub fn request_loading(target: LoadingTarget, request: &mut LoadingRequest) {
    request.target = target;
}

pub fn world_bootstrap_requested(bootstrap: Res<WorldBootstrap>) -> bool {
    bootstrap.requested && !bootstrap.built
}

pub fn finish_world_bootstrap(mut bootstrap: ResMut<WorldBootstrap>) {
    bootstrap.built = true;
}

pub fn update_loading_progress(
    asset_server: Res<AssetServer>,
    preload: Res<GameplayPreload>,
    mut progress: ResMut<LoadingProgress>,
    mut status_query: Query<&mut Text, With<LoadingStatusText>>,
    request: Res<LoadingRequest>,
    mut bootstrap: ResMut<WorldBootstrap>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let loaded = preload
        .handles
        .iter()
        .filter(|handle| asset_server.is_loaded_with_dependencies(handle.id()))
        .count();
    let failed = preload
        .handles
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)));

    for mut text in status_query.iter_mut() {
        **text = if failed {
            "Falha ao carregar um recurso obrigatório".to_string()
        } else if loaded < preload.handles.len() {
            format!("CARREGANDO RECURSOS {loaded}/{}", preload.handles.len())
        } else if !bootstrap.built {
            "MONTANDO A MASMORRA".to_string()
        } else {
            "PRONTO".to_string()
        };
    }

    if failed {
        return;
    }

    if loaded == preload.handles.len() && !bootstrap.built {
        bootstrap.requested = true;
    }

    if bootstrap.built {
        progress.ready_frames = progress.ready_frames.saturating_add(1);
        if progress.ready_frames >= 2 {
            next_state.set(match request.target {
                LoadingTarget::Menu => GameState::MainMenu,
                LoadingTarget::InGame => GameState::InGame,
            });
        }
    }
}

fn begin_preload(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameplayPreload {
        handles: PRELOAD_ASSETS
            .iter()
            .map(|path| asset_server.load_untyped(*path).untyped())
            .collect(),
    });
}

fn setup_loading_overlay(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            LoadingOverlay,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                display: Display::None,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ZIndex(500),
            BackgroundColor(Color::srgb(0.008, 0.018, 0.04)),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("CARREGANDO RECURSOS"),
                TextFont {
                    font: asset_server.load("fonts/DejaVuSans.ttf"),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.42, 0.94, 1.0)),
                LoadingStatusText,
            ));
        });
}

fn show_loading_overlay(
    mut progress: ResMut<LoadingProgress>,
    mut overlay_query: Query<&mut Node, With<LoadingOverlay>>,
) {
    progress.ready_frames = 0;
    for mut node in overlay_query.iter_mut() {
        node.display = Display::Flex;
    }
}

fn hide_loading_overlay(mut overlay_query: Query<&mut Node, With<LoadingOverlay>>) {
    for mut node in overlay_query.iter_mut() {
        node.display = Display::None;
    }
}
