use crate::{
    player::camera::{LocomotionState, Player, PlayerCamera},
    ui::menu::{GameState, MenuNavigation, MenuView},
};
use bevy::{prelude::*, render::view::RenderLayers};

const CINEMATIC_ORIGIN: Vec3 = Vec3::new(64.0, 0.0, 64.0);
const FLOWER_PLAYER_POSITION: Vec3 = Vec3::new(64.0, 1.2, 64.0);
const RIFT_PLAYER_POSITION: Vec3 = Vec3::new(64.0, 1.2, 82.0);
const WATERFALL_TOP_POSITION: Vec3 = Vec3::new(64.0, 10.0, 100.0);
const CONTROL_START_POSITION: Vec3 = Vec3::new(0.0, 1.2, -2.2);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IntroCinematicStage {
    #[default]
    Flowers,
    Rift,
    WaterfallFall,
}

#[derive(Resource, Debug, Default)]
pub struct IntroCinematicState {
    pub requested: bool,
    pub active: bool,
    pub stage: IntroCinematicStage,
    pub elapsed_secs: f32,
}

#[derive(Component)]
struct IntroCinematicOverlay;

#[derive(Component)]
struct IntroCinematicCamera;

#[derive(Component)]
struct IntroCinematicScenery;

pub struct IntroCinematicPlugin;

impl Plugin for IntroCinematicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IntroCinematicState>()
            .add_systems(Startup, setup_intro_cinematic_environment)
            .add_systems(
                Update,
                prepare_intro_scene_layers
                    .before(begin_intro_cinematic)
                    .before(advance_intro_cinematic),
            )
            .add_systems(
                Update,
                (begin_intro_cinematic, advance_intro_cinematic)
                    .chain()
                    .after(crate::player::controller::player_camera_effects)
                    .after(crate::player::controller::update_held_potion_visuals),
            );
    }
}

fn setup_intro_cinematic_environment(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("Intro_Cinematic_Blend_Scenery"),
        IntroCinematicScenery,
        SceneRoot(asset_server.load("models/intro_cinematic.glb#Scene0")),
        Transform::from_translation(CINEMATIC_ORIGIN),
    ));
    let cinematic_camera = commands
        .spawn((
            IntroCinematicCamera,
            Camera3d::default(),
            Camera {
                is_active: false,
                ..default()
            },
            RenderLayers::layer(3),
            Transform::from_translation(CINEMATIC_ORIGIN + Vec3::new(-7.2, 4.6, 8.8))
                .looking_at(CINEMATIC_ORIGIN, Vec3::Y),
            DistanceFog {
                color: Color::srgba(0.025, 0.05, 0.08, 1.0),
                falloff: FogFalloff::ExponentialSquared { density: 0.007 },
                ..default()
            },
        ))
        .id();
    commands.spawn((
        PointLight {
            color: Color::srgb(0.16, 0.92, 0.82),
            intensity: 42000.0,
            range: 20.0,
            shadows_enabled: false,
            ..default()
        },
        RenderLayers::layer(3),
        Transform::from_translation(CINEMATIC_ORIGIN + Vec3::new(-3.0, 6.0, 3.0)),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.70, 0.16, 0.95),
            intensity: 36000.0,
            range: 16.0,
            shadows_enabled: false,
            ..default()
        },
        RenderLayers::layer(3),
        Transform::from_translation(CINEMATIC_ORIGIN + Vec3::new(0.0, 4.0, 18.0)),
    ));
    commands
        .spawn((
            IntroCinematicOverlay,
            TargetCamera(cinematic_camera),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                display: Display::None,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                padding: UiRect::bottom(Val::Px(38.0)),
                ..default()
            },
            ZIndex(300),
        ))
        .with_children(|overlay| {
            overlay.spawn((
                Text::new("Espaço ou Enter para Pular"),
                TextFont {
                    font: asset_server.load("fonts/DejaVuSans.ttf"),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgba(0.90, 0.98, 1.0, 0.92)),
                Node {
                    padding: UiRect::axes(Val::Px(18.0), Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.01, 0.03, 0.06, 0.72)),
                BorderRadius::all(Val::Px(8.0)),
            ));
        });
}

fn prepare_intro_scene_layers(
    mut commands: Commands,
    scenery_query: Query<Entity, With<IntroCinematicScenery>>,
    children_query: Query<&Children>,
) {
    for scenery in scenery_query.iter() {
        apply_layer_to_scene_descendants(
            scenery,
            RenderLayers::layer(3),
            &mut commands,
            &children_query,
        );
    }
}

fn apply_layer_to_scene_descendants(
    entity: Entity,
    layer: RenderLayers,
    commands: &mut Commands,
    children_query: &Query<&Children>,
) {
    commands.entity(entity).insert(layer.clone());
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            apply_layer_to_scene_descendants(*child, layer.clone(), commands, children_query);
        }
    }
}

fn begin_intro_cinematic(
    mut cinematic: ResMut<IntroCinematicState>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut player_query: Query<
        (&mut Transform, &mut LocomotionState),
        (With<Player>, Without<IntroCinematicCamera>),
    >,
    mut cinematic_camera_query: Query<(&mut Transform, &mut Camera), With<IntroCinematicCamera>>,
    mut player_camera_query: Query<
        &mut Camera,
        (With<PlayerCamera>, Without<IntroCinematicCamera>),
    >,
    mut overlay_query: Query<&mut Node, With<IntroCinematicOverlay>>,
) {
    if !cinematic.requested || cinematic.active {
        return;
    }

    cinematic.requested = false;
    cinematic.active = true;
    cinematic.stage = IntroCinematicStage::Flowers;
    cinematic.elapsed_secs = 0.0;
    virtual_time.unpause();

    for mut player_camera in player_camera_query.iter_mut() {
        player_camera.is_active = false;
    }
    for (_, mut cinematic_camera) in cinematic_camera_query.iter_mut() {
        cinematic_camera.is_active = true;
    }
    for mut overlay_node in overlay_query.iter_mut() {
        overlay_node.display = Display::Flex;
    }

    apply_stage_pose(
        IntroCinematicStage::Flowers,
        &mut player_query,
        &mut cinematic_camera_query,
    );
}

fn advance_intro_cinematic(
    time: Res<Time>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut cinematic: ResMut<IntroCinematicState>,
    mut player_query: Query<
        (&mut Transform, &mut LocomotionState),
        (With<Player>, Without<IntroCinematicCamera>),
    >,
    mut cinematic_camera_query: Query<(&mut Transform, &mut Camera), With<IntroCinematicCamera>>,
    mut player_camera_query: Query<
        &mut Camera,
        (With<PlayerCamera>, Without<IntroCinematicCamera>),
    >,
    mut overlay_query: Query<&mut Node, With<IntroCinematicOverlay>>,
    mut nav: ResMut<MenuNavigation>,
    mut loading_request: ResMut<crate::loading::LoadingRequest>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !cinematic.active {
        return;
    }

    let skip_requested = keys
        .as_ref()
        .is_some_and(|keys| keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter));
    if skip_requested {
        finish_intro_cinematic(
            &mut cinematic,
            &mut player_query,
            &mut cinematic_camera_query,
            &mut player_camera_query,
            &mut overlay_query,
            &mut nav,
            &mut loading_request,
            &mut next_state,
        );
        return;
    }

    cinematic.elapsed_secs += time.delta_secs();
    match cinematic.stage {
        IntroCinematicStage::Flowers if cinematic.elapsed_secs >= 4.0 => {
            cinematic.stage = IntroCinematicStage::Rift;
            cinematic.elapsed_secs = 0.0;
            apply_stage_pose(
                cinematic.stage,
                &mut player_query,
                &mut cinematic_camera_query,
            );
        }
        IntroCinematicStage::Rift if cinematic.elapsed_secs >= 4.0 => {
            cinematic.stage = IntroCinematicStage::WaterfallFall;
            cinematic.elapsed_secs = 0.0;
            apply_stage_pose(
                cinematic.stage,
                &mut player_query,
                &mut cinematic_camera_query,
            );
        }
        IntroCinematicStage::WaterfallFall => {
            let mut reached_water = false;
            if let Ok((mut player_transform, mut locomotion)) = player_query.get_single_mut() {
                player_transform.translation.y =
                    WATERFALL_TOP_POSITION.y - cinematic.elapsed_secs * 3.2;
                locomotion.vertical_velocity = -3.2;
                locomotion.is_grounded = false;
                reached_water = player_transform.translation.y <= 0.25;
            }
            if reached_water {
                finish_intro_cinematic(
                    &mut cinematic,
                    &mut player_query,
                    &mut cinematic_camera_query,
                    &mut player_camera_query,
                    &mut overlay_query,
                    &mut nav,
                    &mut loading_request,
                    &mut next_state,
                );
            }
        }
        _ => {}
    }
}

fn apply_stage_pose(
    stage: IntroCinematicStage,
    player_query: &mut Query<
        (&mut Transform, &mut LocomotionState),
        (With<Player>, Without<IntroCinematicCamera>),
    >,
    cinematic_camera_query: &mut Query<(&mut Transform, &mut Camera), With<IntroCinematicCamera>>,
) {
    let Ok((mut player_transform, mut locomotion)) = player_query.get_single_mut() else {
        return;
    };
    let Ok((mut camera_transform, _)) = cinematic_camera_query.get_single_mut() else {
        return;
    };

    locomotion.horizontal_velocity = Vec2::ZERO;
    locomotion.vertical_velocity = 0.0;
    locomotion.is_grounded = true;
    player_transform.rotation = Quat::IDENTITY;

    match stage {
        IntroCinematicStage::Flowers => {
            player_transform.translation = FLOWER_PLAYER_POSITION;
            *camera_transform =
                Transform::from_translation(CINEMATIC_ORIGIN + Vec3::new(-6.8, 3.0, 7.4))
                    .looking_at(CINEMATIC_ORIGIN + Vec3::new(0.0, 0.8, 0.3), Vec3::Y);
        }
        IntroCinematicStage::Rift => {
            player_transform.translation = RIFT_PLAYER_POSITION;
            *camera_transform =
                Transform::from_translation(CINEMATIC_ORIGIN + Vec3::new(2.9, 3.0, 23.2))
                    .looking_at(CINEMATIC_ORIGIN + Vec3::new(0.0, 0.1, 18.3), Vec3::Y);
        }
        IntroCinematicStage::WaterfallFall => {
            player_transform.translation = WATERFALL_TOP_POSITION;
            *camera_transform =
                Transform::from_translation(WATERFALL_TOP_POSITION + Vec3::new(0.0, 1.8, 6.5))
                    .looking_at(WATERFALL_TOP_POSITION + Vec3::new(0.0, -2.4, -1.2), Vec3::Y);
        }
    }
}

fn finish_intro_cinematic(
    cinematic: &mut IntroCinematicState,
    player_query: &mut Query<
        (&mut Transform, &mut LocomotionState),
        (With<Player>, Without<IntroCinematicCamera>),
    >,
    cinematic_camera_query: &mut Query<(&mut Transform, &mut Camera), With<IntroCinematicCamera>>,
    player_camera_query: &mut Query<
        &mut Camera,
        (With<PlayerCamera>, Without<IntroCinematicCamera>),
    >,
    overlay_query: &mut Query<&mut Node, With<IntroCinematicOverlay>>,
    nav: &mut MenuNavigation,
    loading_request: &mut crate::loading::LoadingRequest,
    next_state: &mut NextState<GameState>,
) {
    cinematic.active = false;
    cinematic.elapsed_secs = 0.0;
    if let Ok((mut player_transform, mut locomotion)) = player_query.get_single_mut() {
        player_transform.translation = CONTROL_START_POSITION;
        player_transform.rotation = Quat::IDENTITY;
        locomotion.vertical_velocity = 0.0;
        locomotion.horizontal_velocity = Vec2::ZERO;
        locomotion.is_grounded = true;
    }
    for (_, mut cinematic_camera) in cinematic_camera_query.iter_mut() {
        cinematic_camera.is_active = false;
    }
    for mut player_camera in player_camera_query.iter_mut() {
        player_camera.is_active = true;
    }
    for mut overlay_node in overlay_query.iter_mut() {
        overlay_node.display = Display::None;
    }
    nav.has_started = true;
    nav.current_view = MenuView::Main;
    crate::loading::request_loading(crate::loading::LoadingTarget::InGame, loading_request);
    next_state.set(GameState::Loading);
}
