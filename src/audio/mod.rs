pub mod synth;

use bevy::audio::{AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;
use crate::ui::menu::GameSettings;

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundEffectKind {
    PotionThrow,
    PotionShatter,
    WaterSplash,
    TorchIgnite,
    IceFreeze,
    CreatureChirp,
    UiClick,
    WaterDrop,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SoundEffectEvent {
    pub kind: SoundEffectKind,
    pub volume_multiplier: f32,
}

impl SoundEffectEvent {
    pub fn new(kind: SoundEffectKind) -> Self {
        Self {
            kind,
            volume_multiplier: 1.0,
        }
    }

    pub fn with_volume(kind: SoundEffectKind, volume_multiplier: f32) -> Self {
        Self {
            kind,
            volume_multiplier,
        }
    }
}

#[derive(Resource)]
pub struct AudioAssets {
    pub potion_throw: Handle<AudioSource>,
    pub potion_shatter: Handle<AudioSource>,
    pub water_splash: Handle<AudioSource>,
    pub torch_ignite: Handle<AudioSource>,
    pub ice_freeze: Handle<AudioSource>,
    pub creature_chirp: Handle<AudioSource>,
    pub ui_click: Handle<AudioSource>,
    pub water_drop: Handle<AudioSource>,
    pub ambient_music: Handle<AudioSource>,
}

#[derive(Resource)]
pub struct AmbientDripTimer(pub Timer);

#[derive(Component)]
struct AmbientMusicTrack;

impl Default for AmbientDripTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(5.5, TimerMode::Repeating))
    }
}

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        synth::ensure_procedural_audio_assets();

        app.add_event::<SoundEffectEvent>()
            .init_resource::<AmbientDripTimer>()
            .add_systems(
                Startup,
                setup_audio_assets.after(crate::ui::menu::load_game_settings),
            )
            .add_systems(
                Update,
                (
                    handle_sound_effect_events,
                    ambient_cavern_water_drip,
                    sync_ambient_music_volume,
                ),
            );
    }
}

fn setup_audio_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<GameSettings>,
) {
    let ambient_music = asset_server.load("audio/ambient_cavern.ogg");
    commands.spawn((
        AmbientMusicTrack,
        AudioPlayer(ambient_music.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::new(settings.master_volume * 0.12)),
    ));
    commands.insert_resource(AudioAssets {
        potion_throw: asset_server.load("audio/potion_throw.wav"),
        potion_shatter: asset_server.load("audio/potion_shatter.wav"),
        water_splash: asset_server.load("audio/water_splash.wav"),
        torch_ignite: asset_server.load("audio/torch_ignite.wav"),
        ice_freeze: asset_server.load("audio/ice_freeze.wav"),
        creature_chirp: asset_server.load("audio/creature_chirp.wav"),
        ui_click: asset_server.load("audio/ui_click.wav"),
        water_drop: asset_server.load("audio/water_drop.wav"),
        ambient_music,
    });
}

fn sync_ambient_music_volume(
    settings: Res<GameSettings>,
    music_sinks: Query<&AudioSink, With<AmbientMusicTrack>>,
) {
    if !settings.is_changed() {
        return;
    }
    let volume = settings.master_volume * 0.12;
    for sink in &music_sinks {
        sink.set_volume(volume);
    }
}

pub fn handle_sound_effect_events(
    mut events: EventReader<SoundEffectEvent>,
    audio_assets: Option<Res<AudioAssets>>,
    settings: Option<Res<GameSettings>>,
    mut commands: Commands,
) {
    let Some(audio_assets) = audio_assets else {
        return;
    };
    let master_volume = settings.as_ref().map(|s| s.master_volume).unwrap_or(0.8);
    if master_volume <= 0.001 {
        return;
    }

    for event in events.read() {
        let handle = match event.kind {
            SoundEffectKind::PotionThrow => audio_assets.potion_throw.clone(),
            SoundEffectKind::PotionShatter => audio_assets.potion_shatter.clone(),
            SoundEffectKind::WaterSplash => audio_assets.water_splash.clone(),
            SoundEffectKind::TorchIgnite => audio_assets.torch_ignite.clone(),
            SoundEffectKind::IceFreeze => audio_assets.ice_freeze.clone(),
            SoundEffectKind::CreatureChirp => audio_assets.creature_chirp.clone(),
            SoundEffectKind::UiClick => audio_assets.ui_click.clone(),
            SoundEffectKind::WaterDrop => audio_assets.water_drop.clone(),
        };

        let effective_vol = (master_volume * event.volume_multiplier).clamp(0.0, 1.0);
        commands.spawn((
            AudioPlayer(handle),
            PlaybackSettings::DESPAWN.with_volume(Volume::new(effective_vol)),
        ));
    }
}

pub fn ambient_cavern_water_drip(
    time: Res<Time>,
    mut timer: ResMut<AmbientDripTimer>,
    mut sound_events: EventWriter<SoundEffectEvent>,
    current_state: Option<Res<State<crate::ui::menu::GameState>>>,
) {
    let is_ingame = current_state.map(|s| *s.get() == crate::ui::menu::GameState::InGame).unwrap_or(true);
    if !is_ingame {
        return;
    }

    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        sound_events.send(SoundEffectEvent::with_volume(SoundEffectKind::WaterDrop, 0.35));
    }
}
