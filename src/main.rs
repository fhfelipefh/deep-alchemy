use avian3d::prelude::*;
use bevy::{
    app::AppExit,
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
    window::{CursorGrabMode, PresentMode, WindowResolution},
};

use deep_alchemy::{
    alchemy::AlchemyPlugin,
    dungeon::{DungeonPlugin, SecretTestRoomPlugin},
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
    let is_test_room_screenshot = args.iter().any(|arg| arg == "--screenshot-test-room");
    let is_panorama = args.iter().any(|arg| arg == "--panorama" || arg == "--screenshot-panorama");

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
        .add_plugins(SecretTestRoomPlugin)
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
    } else if is_test_room_screenshot {
        info!(">>> [TEST ROOM SCREENSHOT] Capturing secret white test room frame to ./debug_output/test_room_check.png");
        app.add_systems(Startup, switch_to_ingame_for_test_room_screenshot)
            .add_systems(Update, handle_test_room_screenshot_lifecycle);
    } else if is_panorama {
        info!(">>> [PANORAMA] Capturing 360 degree panoramic view of grotto cavern...");
        app.add_systems(Startup, switch_to_ingame_for_screenshot_test)
            .add_systems(Update, handle_panorama_lifecycle);
    }

    app.run();

    if args.iter().any(|arg| arg == "--validate-frame") {
        let frame_path = std::path::Path::new("debug_output/frame_check.png");
        if frame_path.exists() {
            match deep_alchemy::tools::validator::analyze_frame(frame_path) {
                Ok(report) => {
                    if !report.passed {
                        std::process::exit(1);
                    }
                }
                Err(err) => {
                    eprintln!("Rust validator error: {}", err);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn switch_to_ingame_for_test_room_screenshot(
    mut next_state: ResMut<NextState<GameState>>,
    mut request: ResMut<deep_alchemy::dungeon::test_room::SecretTestRoomRequest>,
) {
    request.requested = true;
    next_state.set(GameState::InGame);
}

fn handle_test_room_screenshot_lifecycle(
    mut commands: Commands,
    mut counter: Local<u32>,
    mut app_exit: EventWriter<AppExit>,
) {
    *counter += 1;
    if *counter == 30 {
        std::fs::create_dir_all("debug_output").ok();
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("debug_output/test_room_check.png"));
        info!(">>> [TEST ROOM SCREENSHOT] Triggered screenshot at frame 30 -> debug_output/test_room_check.png");
    } else if *counter >= 60 {
        info!(">>> [TEST ROOM SCREENSHOT] Frame 60 reached, gracefully exiting.");
        app_exit.send(AppExit::Success);
    }
}

fn switch_to_ingame_for_screenshot_test(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::InGame);
}

fn handle_screenshot_test_lifecycle(
    mut commands: Commands,
    mut counter: Local<u32>,
    mut app_exit: EventWriter<AppExit>,
    mut player_query: Query<&mut Transform, With<deep_alchemy::player::camera::Player>>,
    shards: Query<(&Transform, &deep_alchemy::alchemy::GlassShardParticle), Without<deep_alchemy::player::camera::Player>>,
) {
    *counter += 1;
    if *counter == 2 {
        commands.spawn((
            Transform::from_xyz(0.0, 2.0, -6.5),
            deep_alchemy::alchemy::PotionProjectile {
                velocity: Vec3::new(0.0, -16.0, 0.0),
                effects: vec![deep_alchemy::alchemy::ActiveEffect {
                    kind: deep_alchemy::alchemy::AlchemicalEffectKind::ThermalHeat,
                    duration: 6.0,
                    elapsed: 0.0,
                    magnitude: 35.0,
                }],
                lifetime: 5.0,
                color: Color::srgb(0.98, 0.22, 0.28),
                slot: 1,
                hitstop_timer: 0.0,
                entered_water: false,
            },
        ));
    } else if *counter == 20 {
        info!(">>> [DEBUG] Glass shard count at frame 20: {}", shards.iter().count());
        for (tf, shard) in shards.iter() {
            info!(">>> [DEBUG] Glass shard pos: {:?}, settled: {}, ground_y: {}", tf.translation, shard.settled, shard.ground_y);
        }
        std::fs::create_dir_all("debug_output").ok();
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("debug_output/frame_check.png"));
        info!(">>> [SCREENSHOT GATE] Triggered primary_window screenshot at frame 20 -> debug_output/frame_check.png");
    } else if *counter == 25 {
        if let Ok(mut transform) = player_query.get_single_mut() {
            transform.translation = Vec3::new(0.0, 1.40, -2.8);
            transform.rotation = Quat::from_rotation_y(std::f32::consts::PI) * Quat::from_rotation_x(-0.12);
            info!(">>> [SCREENSHOT GATE] Rotated player 180° to capture rear portal view.");
        }
    } else if *counter == 45 {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("debug_output/frame_rear_check.png"));
        info!(">>> [SCREENSHOT GATE] Triggered rear screenshot at frame 45 -> debug_output/frame_rear_check.png");
    } else if *counter == 48 {
        if let Ok(mut transform) = player_query.get_single_mut() {
            transform.translation = Vec3::new(0.0, 1.40, 7.5);
            transform.rotation = Quat::from_rotation_x(0.48);
            info!(">>> [SCREENSHOT GATE] Moved player inside corridor looking up at ceiling.");
        }
    } else if *counter == 60 {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("debug_output/frame_ceiling_check.png"));
        info!(">>> [SCREENSHOT GATE] Triggered ceiling screenshot at frame 60 -> debug_output/frame_ceiling_check.png");
    } else if *counter >= 80 {
        info!(">>> [SCREENSHOT GATE] Frame 80 reached, gracefully exiting.");
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

fn handle_panorama_lifecycle(
    mut commands: Commands,
    mut counter: Local<u32>,
    mut slice_idx: Local<usize>,
    mut app_exit: EventWriter<AppExit>,
    mut player_query: Query<&mut Transform, With<deep_alchemy::player::camera::Player>>,
    mut hud_query: Query<&mut Visibility, (With<deep_alchemy::ui::hud::HudRoot>, Without<deep_alchemy::player::camera::FirstPersonHeldPotion>)>,
    mut potion_query: Query<&mut Visibility, With<deep_alchemy::player::camera::FirstPersonHeldPotion>>,
) {
    *counter += 1;

    for mut vis in hud_query.iter_mut() {
        if *vis != Visibility::Hidden {
            *vis = Visibility::Hidden;
        }
    }

    for mut vis in potion_query.iter_mut() {
        if *vis != Visibility::Hidden {
            *vis = Visibility::Hidden;
        }
    }

    const WARMUP_FRAMES: u32 = 25;
    const TOTAL_SLICES: usize = 120;
    const FRAMES_PER_SLICE: u32 = 2;

    if *counter == 1 {
        std::fs::remove_dir_all("debug_output/panorama_slices").ok();
        std::fs::create_dir_all("debug_output/panorama_slices").ok();
    }

    if *counter < WARMUP_FRAMES {
        return;
    }

    let progress_frame = *counter - WARMUP_FRAMES;
    let step_in_slice = progress_frame % FRAMES_PER_SLICE;

    const POST_CAPTURE_WAIT_FRAMES: u32 = 60;
    let total_capture_frames = TOTAL_SLICES as u32 * FRAMES_PER_SLICE;

    if *slice_idx < TOTAL_SLICES {
        if step_in_slice == 0 {
            let angle = -(*slice_idx as f32 / TOTAL_SLICES as f32) * std::f32::consts::TAU;
            if let Ok(mut transform) = player_query.get_single_mut() {
                transform.rotation = Quat::from_rotation_y(angle);
            }
        } else if step_in_slice == (FRAMES_PER_SLICE - 1) {
            let path = format!("debug_output/panorama_slices/slice_{:03}.png", *slice_idx);
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path));
            *slice_idx += 1;
        }
    } else if progress_frame == total_capture_frames + POST_CAPTURE_WAIT_FRAMES {
        info!(">>> [PANORAMA] All {} slices captured and flushed. Stitching continuous smartphone-grade panorama...", TOTAL_SLICES);
        if let Err(e) = stitch_panorama(TOTAL_SLICES, "debug_output/panorama_360.png") {
            warn!("Failed to stitch panorama: {}", e);
        } else {
            info!(">>> [PANORAMA] Successfully generated debug_output/panorama_360.png!");
        }
    } else if progress_frame >= total_capture_frames + POST_CAPTURE_WAIT_FRAMES + 10 {
        info!(">>> [PANORAMA] 360° Panorama capture completed. Gracefully exiting.");
        app_exit.send(AppExit::Success);
    }
}

fn sample_bilinear(img: &image::RgbaImage, x: f32, y: f32) -> [f32; 3] {
    let w = img.width();
    let h = img.height();
    let cx = x.clamp(0.0, w as f32 - 1.001);
    let cy = y.clamp(0.0, h as f32 - 1.001);
    let x0 = cx.floor() as u32;
    let y0 = cy.floor() as u32;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let fx = cx - x0 as f32;
    let fy = cy - y0 as f32;

    let p00 = img.get_pixel(x0, y0);
    let p10 = img.get_pixel(x1, y0);
    let p01 = img.get_pixel(x0, y1);
    let p11 = img.get_pixel(x1, y1);

    let mut out = [0.0f32; 3];
    for c in 0..3 {
        let top = p00[c] as f32 * (1.0 - fx) + p10[c] as f32 * fx;
        let bot = p01[c] as f32 * (1.0 - fx) + p11[c] as f32 * fx;
        out[c] = top * (1.0 - fy) + bot * fy;
    }
    out
}

fn stitch_panorama(slice_count: usize, output_path: &str) -> Result<(), String> {
    let mut slices: Vec<image::RgbaImage> = Vec::with_capacity(slice_count);

    for i in 0..slice_count {
        let path = format!("debug_output/panorama_slices/slice_{:03}.png", i);
        let mut attempts = 0;
        let bytes = loop {
            match std::fs::read(&path) {
                Ok(b) if !b.is_empty() => break b,
                _ => {
                    attempts += 1;
                    if attempts > 60 {
                        return Err(format!("Timeout reading slice {}", path));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        };
        let dyn_img = image::load_from_memory(&bytes)
            .map_err(|e| format!("Failed to decode {}: {}", path, e))?;
        slices.push(dyn_img.to_rgba8());
    }

    if slices.is_empty() {
        return Err("No slices provided for panorama".to_string());
    }

    let slice_w = slices[0].width() as f32;
    let slice_h = slices[0].height() as f32;

    let fov_y = 75.0_f32.to_radians();
    let f = (slice_h * 0.5) / (fov_y * 0.5).tan();

    let total_w = (f * std::f32::consts::TAU).round() as usize;
    let out_h = slice_h as usize;

    let cx = slice_w * 0.5;
    let cy = slice_h * 0.5;

    let mut panorama = image::RgbaImage::new(total_w as u32, out_h as u32);
    let tau_over_n = std::f32::consts::TAU / slice_count as f32;

    for x in 0..total_w {
        let slice_pos = (x as f32 / total_w as f32) * slice_count as f32;
        let k0 = slice_pos.floor() as usize % slice_count;
        let k1 = (k0 + 1) % slice_count;
        let t = slice_pos - slice_pos.floor();

        let theta0 = t * tau_over_n;
        let theta1 = (t - 1.0) * tau_over_n;

        let src_x0 = cx + f * theta0.tan();
        let src_x1 = cx + f * theta1.tan();

        let sec0 = (1.0 + theta0.tan() * theta0.tan()).sqrt();
        let sec1 = (1.0 + theta1.tan() * theta1.tan()).sqrt();

        let w1 = 0.5 * (1.0 - (t * std::f32::consts::PI).cos());
        let w0 = 1.0 - w1;

        let img0 = &slices[k0];
        let img1 = &slices[k1];

        for y in 0..out_h {
            let y_cyl = y as f32 - cy;
            let src_y0 = cy + y_cyl * sec0;
            let src_y1 = cy + y_cyl * sec1;

            let c0 = sample_bilinear(img0, src_x0, src_y0);
            let c1 = sample_bilinear(img1, src_x1, src_y1);

            let r = (c0[0] * w0 + c1[0] * w1).round().clamp(0.0, 255.0) as u8;
            let g = (c0[1] * w0 + c1[1] * w1).round().clamp(0.0, 255.0) as u8;
            let b = (c0[2] * w0 + c1[2] * w1).round().clamp(0.0, 255.0) as u8;

            panorama.put_pixel(x as u32, y as u32, image::Rgba([r, g, b, 255]));
        }
    }

    panorama
        .save(output_path)
        .map_err(|e| format!("Failed to save panorama to {}: {}", output_path, e))?;

    Ok(())
}
