use avian3d::prelude::*;
use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
    window::{CursorGrabMode, PresentMode, WindowResolution},
};

use deep_alchemy::{
    alchemy::AlchemyPlugin,
    dungeon::DungeonPlugin,
    player::PlayerPlugin,
    render::RenderPipelinePlugin,
    ui::{menu::GameState, HudPlugin, MenuPlugin},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let is_screenshot_test = args
        .iter()
        .any(|arg| arg == "--screenshot-test" || arg == "--validate-frame");
    let is_menu_screenshot = args.iter().any(|arg| arg == "--screenshot-menu");
    let is_settings_screenshot = args.iter().any(|arg| arg == "--screenshot-settings");

    let mut app = App::new();

    app.insert_resource(ClearColor(Color::srgb(0.02, 0.03, 0.05)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Deep Alchemy".to_string(),
                resolution: WindowResolution::new(1280.0, 720.0),
                present_mode: PresentMode::AutoVsync,
                cursor_options: bevy::window::CursorOptions {
                    grab_mode: CursorGrabMode::None,
                    visible: true,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(RenderPipelinePlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(DungeonPlugin)
        .add_plugins(AlchemyPlugin)
        .add_plugins(HudPlugin)
        .add_plugins(MenuPlugin);

    if is_screenshot_test {
        info!(">>> [GATE 4] Running in --screenshot-test mode: capturing frame 20 and dumping telemetry");
        app.add_systems(Startup, switch_to_ingame_for_screenshot_test)
            .add_systems(
                Update,
                (
                    handle_screenshot_test_lifecycle,
                    deep_alchemy::dungeon::dump_camera_frustum_telemetry,
                ),
            );
    } else if is_menu_screenshot {
        info!(">>> [MENU SCREENSHOT] Capturing main menu frame to ./debug_output/menu_check.png");
        app.add_systems(Update, handle_menu_screenshot_lifecycle);
    } else if is_settings_screenshot {
        info!(">>> [SETTINGS SCREENSHOT] Capturing settings subview frame to ./debug_output/settings_check.png");
        app.add_systems(Update, handle_settings_screenshot_lifecycle);
    }

    app.run();
}

fn switch_to_ingame_for_screenshot_test(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::InGame);
}

fn handle_screenshot_test_lifecycle(
    mut commands: Commands,
    mut counter: Local<u32>,
    mut app_exit: EventWriter<AppExit>,
    mut player_query: Query<&mut Transform, With<deep_alchemy::player::camera::Player>>,
) {
    *counter += 1;
    if *counter == 20 {
        std::fs::create_dir_all("debug_output").ok();
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("debug_output/frame_check.png"));
        info!(">>> [SCREENSHOT GATE] Triggered primary_window screenshot at frame 20 -> debug_output/frame_check.png");
    } else if *counter == 25 {
        if let Ok(mut transform) = player_query.get_single_mut() {
            transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
            info!(">>> [SCREENSHOT GATE] Rotated player 180° to capture rear portal view.");
        }
    } else if *counter == 45 {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("debug_output/frame_rear_check.png"));
        info!(">>> [SCREENSHOT GATE] Triggered rear screenshot at frame 45 -> debug_output/frame_rear_check.png");
    } else if *counter >= 70 {
        info!(">>> [SCREENSHOT GATE] Frame 70 reached, gracefully exiting.");
        app_exit.send(AppExit::Success);
    }
}

fn handle_menu_screenshot_lifecycle(
    mut commands: Commands,
    mut counter: Local<u32>,
    mut app_exit: EventWriter<AppExit>,
) {
    *counter += 1;
    if *counter == 20 {
        std::fs::create_dir_all("debug_output").ok();
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("debug_output/menu_check.png"));
        info!(">>> [MENU SCREENSHOT] Triggered primary_window screenshot at frame 20 -> debug_output/menu_check.png");
    } else if *counter >= 50 {
        info!(">>> [MENU SCREENSHOT] Frame 50 reached, gracefully exiting.");
        app_exit.send(AppExit::Success);
    }
}

fn handle_settings_screenshot_lifecycle(
    mut commands: Commands,
    mut counter: Local<u32>,
    mut app_exit: EventWriter<AppExit>,
    mut nav: ResMut<deep_alchemy::ui::menu::MenuNavigation>,
) {
    *counter += 1;
    if *counter == 5 {
        nav.current_view = deep_alchemy::ui::menu::MenuView::Settings;
    }
    if *counter == 25 {
        std::fs::create_dir_all("debug_output").ok();
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("debug_output/settings_check.png"));
        info!(">>> [SETTINGS SCREENSHOT] Triggered primary_window screenshot at frame 25 -> debug_output/settings_check.png");
    } else if *counter >= 50 {
        info!(">>> [SETTINGS SCREENSHOT] Frame 50 reached, gracefully exiting.");
        app_exit.send(AppExit::Success);
    }
}
