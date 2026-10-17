mod alchemy;
mod dungeon;
mod player;
mod render;
mod ui;

use avian3d::prelude::*;
use bevy::{
    prelude::*,
    window::{CursorGrabMode, PresentMode, WindowResolution},
};

use alchemy::AlchemyPlugin;
use dungeon::DungeonPlugin;
use player::PlayerPlugin;
use render::RenderPipelinePlugin;
use ui::HudPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Deep Alchemy".to_string(),
                    resolution: WindowResolution::new(1280.0, 720.0),
                    present_mode: PresentMode::AutoVsync,
                    cursor_options: bevy::window::CursorOptions {
                        grab_mode: CursorGrabMode::Locked,
                        visible: false,
                        ..default()
                    },
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(RenderPipelinePlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(DungeonPlugin)
        .add_plugins(AlchemyPlugin)
        .add_plugins(HudPlugin)
        .run();
}
