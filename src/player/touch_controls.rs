use crate::ui::menu::GameState;
use bevy::prelude::*;

pub const COLOR_TOUCH_BG: Color = Color::srgba(0.04, 0.06, 0.10, 0.55);
pub const COLOR_TOUCH_BG_PRESSED: Color = Color::srgba(0.18, 0.38, 0.48, 0.85);
pub const COLOR_TOUCH_BORDER: Color = Color::srgba(0.82, 0.68, 0.32, 0.75);
pub const COLOR_TOUCH_BORDER_ACTIVE: Color = Color::srgba(0.98, 0.85, 0.35, 1.0);

#[derive(Resource, Debug, Clone)]
pub struct TouchControlsState {
    pub enabled: bool,
    pub joystick_active: bool,
    pub joystick_touch_id: Option<u64>,
    pub joystick_origin: Vec2,
    pub joystick_current: Vec2,
    pub move_axis: Vec2,
    pub is_sprinting: bool,
    pub look_touch_id: Option<u64>,
    pub last_look_pos: Vec2,
    pub look_delta: Vec2,
    pub jump_pressed: bool,
    pub jump_just_pressed: bool,
    pub crouch_pressed: bool,
    pub interact_just_pressed: bool,
    pub throw_just_pressed: bool,
    pub aim_pressed: bool,
    pub toggle_menu_just_pressed: bool,
    pub sensitivity: f32,
}

impl Default for TouchControlsState {
    fn default() -> Self {
        let is_mobile = cfg!(target_os = "android") || cfg!(target_os = "ios");
        Self {
            enabled: is_mobile,
            joystick_active: false,
            joystick_touch_id: None,
            joystick_origin: Vec2::ZERO,
            joystick_current: Vec2::ZERO,
            move_axis: Vec2::ZERO,
            is_sprinting: false,
            look_touch_id: None,
            last_look_pos: Vec2::ZERO,
            look_delta: Vec2::ZERO,
            jump_pressed: false,
            jump_just_pressed: false,
            crouch_pressed: false,
            interact_just_pressed: false,
            throw_just_pressed: false,
            aim_pressed: false,
            toggle_menu_just_pressed: false,
            sensitivity: 0.0035,
        }
    }
}

#[derive(Component)]
pub struct TouchControlsRoot;

#[derive(Component)]
pub struct VirtualJoystickBase;

#[derive(Component)]
pub struct VirtualJoystickKnob;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchAction {
    Jump,
    Crouch,
    Interact,
    Throw,
    Aim,
    Pause,
}

#[derive(Component)]
pub struct TouchActionButton {
    pub action: TouchAction,
}

pub struct TouchControlsPlugin;

impl Plugin for TouchControlsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TouchControlsState>()
            .add_systems(Startup, setup_touch_controls_ui)
            .add_systems(
                Update,
                (
                    process_touch_inputs,
                    update_touch_controls_visibility,
                    handle_touch_button_visuals,
                ),
            );
    }
}

pub fn setup_touch_controls_ui(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    touch_state: Res<TouchControlsState>,
) {
    let font_handle: Handle<Font> = asset_server
        .as_ref()
        .map(|s| s.load("fonts/DejaVuSans.ttf"))
        .unwrap_or_default();

    let initial_visibility = if touch_state.enabled {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    commands
        .spawn((
            TouchControlsRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            initial_visibility,
            ZIndex(15),
        ))
        .with_children(|root| {
            root.spawn((
                VirtualJoystickBase,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(36.0),
                    bottom: Val::Px(36.0),
                    width: Val::Px(120.0),
                    height: Val::Px(120.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderRadius::all(Val::Px(60.0)),
                BackgroundColor(COLOR_TOUCH_BG),
                BorderColor(COLOR_TOUCH_BORDER),
            ))
            .with_children(|base| {
                base.spawn((
                    VirtualJoystickKnob,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(35.0),
                        top: Val::Px(35.0),
                        width: Val::Px(50.0),
                        height: Val::Px(50.0),
                        border: UiRect::all(Val::Px(1.5)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(25.0)),
                    BackgroundColor(Color::srgba(0.12, 0.20, 0.28, 0.85)),
                    BorderColor(Color::srgba(0.82, 0.68, 0.32, 0.90)),
                ));
            });

            spawn_touch_button(
                root,
                font_handle.clone(),
                TouchAction::Jump,
                "▲",
                64.0,
                Val::Px(32.0),
                Val::Px(32.0),
                22.0,
            );

            spawn_touch_button(
                root,
                font_handle.clone(),
                TouchAction::Throw,
                "✦",
                56.0,
                Val::Px(106.0),
                Val::Px(32.0),
                20.0,
            );

            spawn_touch_button(
                root,
                font_handle.clone(),
                TouchAction::Aim,
                "◎",
                52.0,
                Val::Px(106.0),
                Val::Px(98.0),
                18.0,
            );

            spawn_touch_button(
                root,
                font_handle.clone(),
                TouchAction::Interact,
                "E",
                52.0,
                Val::Px(172.0),
                Val::Px(38.0),
                18.0,
            );

            spawn_touch_button(
                root,
                font_handle.clone(),
                TouchAction::Crouch,
                "▼",
                52.0,
                Val::Px(32.0),
                Val::Px(106.0),
                18.0,
            );

            root.spawn((
                TouchActionButton {
                    action: TouchAction::Pause,
                },
                Button,
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(24.0),
                    top: Val::Px(24.0),
                    width: Val::Px(44.0),
                    height: Val::Px(44.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.5)),
                    ..default()
                },
                BorderRadius::all(Val::Px(22.0)),
                BackgroundColor(COLOR_TOUCH_BG),
                BorderColor(COLOR_TOUCH_BORDER),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("❚❚"),
                    TextFont {
                        font: font_handle.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(COLOR_TOUCH_BORDER_ACTIVE),
                ));
            });
        });
}

fn spawn_touch_button(
    parent: &mut ChildBuilder,
    font: Handle<Font>,
    action: TouchAction,
    label: &str,
    size: f32,
    right: Val,
    bottom: Val,
    font_size: f32,
) {
    parent
        .spawn((
            TouchActionButton { action },
            Button,
            Node {
                position_type: PositionType::Absolute,
                right,
                bottom,
                width: Val::Px(size),
                height: Val::Px(size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            BorderRadius::all(Val::Px(size * 0.5)),
            BackgroundColor(COLOR_TOUCH_BG),
            BorderColor(COLOR_TOUCH_BORDER),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font,
                    font_size,
                    ..default()
                },
                TextColor(COLOR_TOUCH_BORDER_ACTIVE),
            ));
        });
}

pub fn process_touch_inputs(
    touches: Option<Res<Touches>>,
    windows: Query<&Window>,
    mut touch_state: ResMut<TouchControlsState>,
    action_buttons: Query<(&Interaction, &TouchActionButton)>,
    mut knob_query: Query<&mut Node, With<VirtualJoystickKnob>>,
) {
    touch_state.jump_just_pressed = false;
    touch_state.interact_just_pressed = false;
    touch_state.throw_just_pressed = false;
    touch_state.toggle_menu_just_pressed = false;
    touch_state.look_delta = Vec2::ZERO;

    let mut jump_now = false;
    let mut crouch_now = false;
    let mut aim_now = false;

    for (interaction, button) in action_buttons.iter() {
        if *interaction == Interaction::Pressed {
            match button.action {
                TouchAction::Jump => {
                    jump_now = true;
                }
                TouchAction::Crouch => {
                    crouch_now = true;
                }
                TouchAction::Interact => {
                    touch_state.interact_just_pressed = true;
                }
                TouchAction::Throw => {
                    touch_state.throw_just_pressed = true;
                }
                TouchAction::Aim => {
                    aim_now = true;
                }
                TouchAction::Pause => {
                    touch_state.toggle_menu_just_pressed = true;
                }
            }
        }
    }

    if jump_now && !touch_state.jump_pressed {
        touch_state.jump_just_pressed = true;
    }
    touch_state.jump_pressed = jump_now;
    touch_state.crouch_pressed = crouch_now;
    touch_state.aim_pressed = aim_now;

    let Some(touches) = touches else {
        return;
    };

    if !touch_state.enabled && touches.iter().count() > 0 {
        touch_state.enabled = true;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let screen_w = window.width();
    let screen_h = window.height();
    let left_zone_limit = screen_w * 0.45;

    if let Some(id) = touch_state.joystick_touch_id {
        if touches.just_released(id) || touches.get_pressed(id).is_none() {
            touch_state.joystick_active = false;
            touch_state.joystick_touch_id = None;
            touch_state.move_axis = Vec2::ZERO;
            touch_state.is_sprinting = false;
            for mut knob in knob_query.iter_mut() {
                knob.left = Val::Px(35.0);
                knob.top = Val::Px(35.0);
            }
        } else if let Some(touch) = touches.get_pressed(id) {
            let cur = touch.position();
            let offset = cur - touch_state.joystick_origin;
            let dist = offset.length();
            const MAX_RADIUS: f32 = 55.0;
            let (dir, len) = if dist > 1e-4 {
                (offset / dist, dist.min(MAX_RADIUS))
            } else {
                (Vec2::ZERO, 0.0)
            };
            let norm_len = len / MAX_RADIUS;
            touch_state.move_axis = Vec2::new(dir.x * norm_len, -dir.y * norm_len);
            touch_state.is_sprinting = dist > 45.0;

            let clamped_offset = dir * len;
            for mut knob in knob_query.iter_mut() {
                knob.left = Val::Px(35.0 + clamped_offset.x);
                knob.top = Val::Px(35.0 + clamped_offset.y);
            }
        }
    } else {
        for touch in touches.iter_just_pressed() {
            let pos = touch.position();
            if pos.x < left_zone_limit && pos.y > screen_h * 0.25 {
                touch_state.joystick_touch_id = Some(touch.id());
                touch_state.joystick_origin = pos;
                touch_state.joystick_current = pos;
                touch_state.joystick_active = true;
                break;
            }
        }
    }

    if let Some(id) = touch_state.look_touch_id {
        if touches.just_released(id) || touches.get_pressed(id).is_none() {
            touch_state.look_touch_id = None;
            touch_state.look_delta = Vec2::ZERO;
        } else if let Some(touch) = touches.get_pressed(id) {
            let cur = touch.position();
            touch_state.look_delta = cur - touch_state.last_look_pos;
            touch_state.last_look_pos = cur;
        }
    } else {
        for touch in touches.iter_just_pressed() {
            let tid = touch.id();
            if Some(tid) == touch_state.joystick_touch_id {
                continue;
            }
            let pos = touch.position();
            let not_in_actions = pos.y < screen_h - 180.0 || pos.x < screen_w - 240.0;
            let not_in_pause = !(pos.x > screen_w - 90.0 && pos.y < 90.0);
            if pos.x >= left_zone_limit && not_in_actions && not_in_pause {
                touch_state.look_touch_id = Some(tid);
                touch_state.last_look_pos = pos;
                touch_state.look_delta = Vec2::ZERO;
                break;
            }
        }
    }
}

pub fn update_touch_controls_visibility(
    game_state: Option<Res<State<GameState>>>,
    touch_state: Res<TouchControlsState>,
    mut root_query: Query<&mut Visibility, With<TouchControlsRoot>>,
) {
    let in_game = game_state.map(|s| *s.get() == GameState::InGame).unwrap_or(true);
    let should_show = in_game && touch_state.enabled;
    for mut vis in root_query.iter_mut() {
        let target_vis = if should_show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *vis != target_vis {
            *vis = target_vis;
        }
    }
}

pub fn handle_touch_button_visuals(
    mut query: Query<(&Interaction, &mut BackgroundColor, &mut BorderColor), With<TouchActionButton>>,
) {
    for (interaction, mut bg, mut border) in query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(COLOR_TOUCH_BG_PRESSED);
                *border = BorderColor(COLOR_TOUCH_BORDER_ACTIVE);
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgba(0.08, 0.14, 0.22, 0.75));
                *border = BorderColor(COLOR_TOUCH_BORDER_ACTIVE);
            }
            Interaction::None => {
                *bg = BackgroundColor(COLOR_TOUCH_BG);
                *border = BorderColor(COLOR_TOUCH_BORDER);
            }
        }
    }
}
