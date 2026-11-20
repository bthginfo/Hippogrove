use std::collections::BTreeMap;

#[cfg(target_os = "android")]
mod android;

#[cfg(not(any(target_os = "ios", target_os = "android")))]
use bevy::input::gamepad::{GamepadAxis, GamepadButton};
use bevy::{
    input::{
        ButtonState,
        gamepad::Gamepad,
        mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel},
        touch::{TouchInput, TouchPhase},
    },
    prelude::*,
    window::{AppLifecycle, PrimaryWindow, WindowFocused},
};

use crate::{
    civic::Civic,
    game::{Building, Game, Miracle},
    hud::{self, HitTest},
    play::{CameraRig, Panel, Play, Tool, UiAction},
    region::{CELL, Material, Tile},
    terrain,
    view::{Board, WorldCamera},
};

const SOUTH: u32 = 1;
const EAST: u32 = 2;
const LEFT: u32 = 4;
const RIGHT: u32 = 8;
const UP: u32 = 16;
const DOWN: u32 = 32;
const MENU: u32 = 64;
const ROTATE: u32 = 128;
const GUIDE: u32 = 256;
const PROGRESS: u32 = 512;

#[derive(Clone, Copy, Default)]
struct Pad {
    left: Vec2,
    right: Vec2,
    zoom: f32,
    buttons: u32,
    pressed: u32,
    connected: bool,
}

#[cfg(any(target_os = "ios", target_os = "android"))]
static MOBILE_PAD: std::sync::Mutex<Pad> = std::sync::Mutex::new(Pad {
    left: Vec2::ZERO,
    right: Vec2::ZERO,
    zoom: 0.,
    buttons: 0,
    pressed: 0,
    connected: false,
});

#[cfg(any(target_os = "ios", target_os = "android"))]
static SAFE_INSETS: std::sync::Mutex<Vec4> = std::sync::Mutex::new(Vec4::ZERO);

pub fn safe_insets(_window: &Window) -> [u16; 4] {
    #[cfg(any(target_os = "ios", target_os = "android"))]
    {
        let insets = *SAFE_INSETS
            .lock()
            .unwrap_or_else(|state| state.into_inner());
        #[cfg(target_os = "android")]
        let insets = insets / _window.scale_factor();
        insets.to_array().map(|inset| inset.max(0.).ceil() as u16)
    }
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    [0; 4]
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_game_hippogrove_GameActivity_publishInsets(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
) {
    *SAFE_INSETS
        .lock()
        .unwrap_or_else(|state| state.into_inner()) =
        Vec4::new(left as f32, top as f32, right as f32, bottom as f32);
}

#[cfg(target_os = "ios")]
#[unsafe(no_mangle)]
pub extern "C" fn hippogrove_safe_insets(left: f32, top: f32, right: f32, bottom: f32) {
    *SAFE_INSETS
        .lock()
        .unwrap_or_else(|state| state.into_inner()) = Vec4::new(left, top, right, bottom);
}

#[cfg(any(target_os = "ios", target_os = "android"))]
fn native_pad(
    left_x: f32,
    left_y: f32,
    right_x: f32,
    right_y: f32,
    zoom: f32,
    buttons: u32,
    connected: bool,
) {
    let mut state = MOBILE_PAD
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *state = if connected {
        Pad {
            left: Vec2::new(left_x, left_y),
            right: Vec2::new(right_x, right_y),
            zoom,
            buttons,
            pressed: state.pressed | (buttons & !state.buttons),
            connected,
        }
    } else {
        Pad::default()
    };
}

#[cfg(target_os = "ios")]
#[unsafe(no_mangle)]
pub extern "C" fn hippogrove_controller_state(
    lx: f32,
    ly: f32,
    rx: f32,
    ry: f32,
    zoom: f32,
    buttons: u32,
    connected: u32,
) {
    native_pad(lx, ly, rx, ry, zoom, buttons, connected != 0);
}

#[cfg(target_os = "ios")]
#[link(name = "HippogroveController", kind = "static")]
unsafe extern "C" {
    fn hippogrove_start_controllers();
}

#[cfg(target_os = "ios")]
#[link(name = "GameController", kind = "framework")]
unsafe extern "C" {}

#[cfg(target_os = "ios")]
#[link(name = "UIKit", kind = "framework")]
unsafe extern "C" {}

#[cfg(target_os = "ios")]
pub fn setup_native() {
    // The shim installs Game Controller callbacks on the Apple main thread.
    unsafe { hippogrove_start_controllers() };
}

fn read_pad(gamepads: &Query<&Gamepad>) -> Pad {
    #[cfg(any(target_os = "ios", target_os = "android"))]
    {
        let _ = gamepads;
        let mut state = MOBILE_PAD
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let snapshot = *state;
        state.pressed = 0;
        snapshot
    }
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        let Some(pad) = gamepads.iter().next() else {
            return Pad::default();
        };
        let pressed = |button, bit| if pad.pressed(button) { bit } else { 0 };
        let axis = |axis| pad.get(axis).unwrap_or(0.);
        Pad {
            left: Vec2::new(axis(GamepadAxis::LeftStickX), axis(GamepadAxis::LeftStickY)),
            right: Vec2::new(
                axis(GamepadAxis::RightStickX),
                axis(GamepadAxis::RightStickY),
            ),
            zoom: pad.get(GamepadButton::RightTrigger2).unwrap_or(0.)
                - pad.get(GamepadButton::LeftTrigger2).unwrap_or(0.),
            buttons: pressed(GamepadButton::South, SOUTH)
                | pressed(GamepadButton::East, EAST)
                | pressed(GamepadButton::DPadLeft, LEFT)
                | pressed(GamepadButton::DPadRight, RIGHT)
                | pressed(GamepadButton::DPadUp, UP)
                | pressed(GamepadButton::DPadDown, DOWN)
                | pressed(GamepadButton::Start, MENU)
                | pressed(GamepadButton::West, ROTATE)
                | pressed(GamepadButton::RightTrigger, GUIDE)
                | pressed(GamepadButton::LeftTrigger, PROGRESS),
            connected: true,
            pressed: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct Drag {
    start: Vec2,
    last: Vec2,
    over_ui: bool,
    pressed: Option<UiAction>,
    dragging: bool,
}

impl Drag {
    fn new(position: Vec2, (over_ui, pressed): (bool, Option<UiAction>)) -> Self {
        Self {
            start: position,
            last: position,
            over_ui,
            pressed,
            dragging: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct Controls {
    fingers: BTreeMap<u64, Vec2>,
    touch: Option<(u64, Drag)>,
    mouse: Option<Drag>,
    pad_buttons: u32,
    resume_pad: bool,
    pinch_in_world: bool,
}

fn ground_point(
    game: &Game,
    camera: &Camera,
    pose: &GlobalTransform,
    screen: Vec2,
) -> Option<Vec3> {
    camera
        .viewport_to_world(pose, screen)
        .ok()
        .and_then(|ray| terrain::ray_contact(game.region(), ray, game.city().bridge_fixed))
}

fn select(play: &mut Play, game: &Game, camera: &Camera, pose: &GlobalTransform, screen: Vec2) {
    let region = game.region();
    let city = game.city();
    let Some(point) = ground_point(game, camera, pose, screen) else {
        return;
    };
    let mut tile = Tile::from_world(point);
    if play.tool == Tool::Build {
        if play.building() == Building::Shrine && tile.within(region.frontier, 1) {
            tile = region.frontier;
        }
    } else {
        let world_scale = camera
            .world_to_viewport(pose, point + Vec3::X * CELL)
            .map_or(19., |other| other.distance(screen));
        let mut best = (world_scale * 0.58).clamp(9., 30.);
        let mut consider = |candidate: Tile, p: Vec3| {
            let y = terrain::actor_height(region, p, city.bridge_fixed);
            if let Ok(projected) = camera.world_to_viewport(pose, p + Vec3::Y * y) {
                let distance = projected.distance(screen);
                if distance < best {
                    best = distance;
                    tile = candidate;
                }
            }
        };
        if play.tool == Tool::Hand
            || (matches!(play.tool, Tool::Helpers | Tool::Hippo) && play.option == 0)
        {
            for (index, source) in region.sources.iter().enumerate() {
                consider(source.tile, source.tile.world() + Vec3::Y * 0.35);
                if index >= region.original_sources && source.material == Material::Wood {
                    consider(source.tile, source.tile.world() + Vec3::Y * 1.54);
                }
            }
        }
        if play.tool == Tool::Hand {
            consider(region.depot, region.depot.world() + Vec3::Y * 0.21);
            consider(region.bridge(), region.bridge().world() + Vec3::Y * 0.23);
            for district in &region.districts {
                consider(district.center, district.center.world() + Vec3::Y * 0.55);
            }
            for area in &region.areas {
                consider(area.center, area.center.world() + Vec3::Y * 0.56);
            }
            for site in &city.buildings {
                if city.held.is_none() || city.can_store_at(region, site.tile) {
                    let center = site.kind.center(site.tile);
                    let adjust = if site.kind == Building::Field {
                        0.19
                    } else {
                        1.05
                    };
                    consider(site.tile, center + Vec3::Y * adjust);
                }
            }
        }
        if play.tool == Tool::Helpers && play.option == 1 {
            for site in &city.buildings {
                if site.kind == Building::Civic(Civic::Barracks) {
                    consider(site.tile, site.kind.center(site.tile) + Vec3::Y * 0.82);
                }
            }
        }
        if (play.tool == Tool::Plan && play.option == 1)
            || (play.tool == Tool::Hippo && play.option == 2)
        {
            for site in &city.buildings {
                if play.tool == Tool::Plan || site.kind == Building::Field {
                    consider(
                        site.tile,
                        site.kind.center(site.tile)
                            + Vec3::Y
                                * if site.kind == Building::Field {
                                    0.2
                                } else {
                                    0.88
                                },
                    );
                }
            }
        }
        if play.tool == Tool::Miracles && play.miracle().offensive() {
            for (index, &spirit) in city.guards.iter().enumerate() {
                if spirit > 0 {
                    let point = city.guard_world(region, index);
                    consider(Tile::from_world(point), point + Vec3::Y * 0.65);
                }
            }
            for (district, state) in region.districts.iter().zip(&city.districts) {
                if state.outcome.is_none() && state.resolve > 0 {
                    consider(district.center, district.center.world() + Vec3::Y * 0.55);
                }
            }
        }
        if play.tool == Tool::Miracles && play.miracle() == Miracle::Mend {
            let hippo = &city.walkers[0];
            consider(hippo.tile(), hippo.world() + Vec3::Y);
        }
    }
    if region.bounds.contains(tile) {
        play.aim = tile;
    }
}

fn drag(gesture: &mut Drag, position: Vec2, play: &Play, view: &mut CameraRig, height: f32) {
    let delta = position - gesture.last;
    if !gesture.dragging && gesture.start.distance(position) > 9. {
        gesture.dragging = true;
    }
    if gesture.dragging && !gesture.over_ui && play.panel == Panel::Closed {
        let scale = view.rendered_height / height.max(120.);
        view.pan(-delta.x * scale, delta.y * scale);
    }
    gesture.last = position;
}

fn release(
    gesture: Drag,
    position: Vec2,
    play: &mut Play,
    game: &mut Game,
    view: &mut CameraRig,
    camera: &Camera,
    pose: &GlobalTransform,
    areas: &HitTest,
    scale: f32,
    now: f32,
) {
    let (over_ui, action) = hud::hit(areas, position, scale);
    if !gesture.dragging && gesture.start.distance(position) <= 15. {
        if gesture.over_ui {
            if let Some(action) = action.filter(|action| Some(*action) == gesture.pressed) {
                play.activate(action, game, view, now);
            }
        } else if !over_ui && play.panel == Panel::Closed {
            let city = game.city();
            let region = game.region();
            if play.tool == Tool::Hand
                && city.held.is_none()
                && view.rendered_height < 46.
                && matches!(
                    city.side_quest(region),
                    crate::game::SideQuest::Available | crate::game::SideQuest::Ready
                )
                && camera
                    .world_to_viewport(pose, crate::life::sign_position(region))
                    .is_ok_and(|tip| tip.distance(position) <= 15.)
            {
                play.activate(UiAction::Panel(Panel::SideQuest), game, view, now);
                return;
            }
            let hippo = city.walkers[0].world();
            let ground = terrain::actor_height(game.region(), hippo, city.bridge_fixed);
            if play.tool == Tool::Hand
                && city.held.is_none()
                && view.rendered_height < 48.
                && camera
                    .world_to_viewport(pose, hippo.with_y(ground + 0.92))
                    .is_ok_and(|tip| tip.distance(position) <= 19.)
            {
                play.aim = city.walkers[0].tile();
                play.activate(UiAction::Tool(Tool::Hippo), game, view, now);
                return;
            }
            select(play, game, camera, pose, position);
        }
    }
}

fn deadzone(stick: Vec2) -> Vec2 {
    let distance = stick.length().min(1.);
    if distance < 0.17 {
        Vec2::ZERO
    } else {
        stick.normalize_or_zero() * ((distance - 0.17) / 0.83).powf(1.45)
    }
}

pub fn collect(
    mut touches: EventReader<TouchInput>,
    mut mouse: EventReader<MouseButtonInput>,
    mut wheel: EventReader<MouseWheel>,
    mut focus: EventReader<WindowFocused>,
    mut lifecycle: EventReader<AppLifecycle>,
    window: Single<(Entity, &Window), With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<WorldCamera>>,
    keys: Res<ButtonInput<KeyCode>>,
    areas: HitTest,
    gamepads: Query<&Gamepad>,
    time: Res<Time>,
    mut controls: ResMut<Controls>,
    mut play: ResMut<Play>,
    mut game: ResMut<Game>,
    mut view: ResMut<CameraRig>,
    board: Res<Board>,
) {
    let (entity, window) = *window;
    let (camera, pose) = *camera;
    let now = time.elapsed_secs();
    let dt = time.delta_secs().min(0.08);
    let scale = window.scale_factor();
    let size = Vec2::new(window.width(), window.height());
    for event in lifecycle.read() {
        match event {
            AppLifecycle::WillSuspend | AppLifecycle::Suspended => play.paused = true,
            AppLifecycle::Running => play.paused = false,
            _ => {}
        }
    }
    for event in focus.read().filter(|event| event.window == entity) {
        play.paused = !event.focused;
    }
    if play.paused || board.loading(&game) {
        controls.fingers.clear();
        controls.touch = None;
        controls.mouse = None;
        controls.pinch_in_world = false;
        controls.resume_pad = true;
        touches.clear();
        mouse.clear();
        wheel.clear();
        return;
    }

    let mut touched = false;
    for event in touches.read().filter(|event| event.window == entity) {
        touched = true;
        play.gamepad = false;
        play.pointer = Some(event.position);
        match event.phase {
            TouchPhase::Started => {
                controls.fingers.insert(event.id, event.position);
                let hit = hud::hit(&areas, event.position, scale);
                if controls.fingers.len() == 1 {
                    controls.touch = Some((event.id, Drag::new(event.position, hit)));
                } else {
                    if controls.fingers.len() == 2 {
                        controls.pinch_in_world = !hit.0
                            && controls
                                .touch
                                .as_ref()
                                .is_some_and(|(_, first)| !first.over_ui);
                    }
                    controls.touch = None;
                }
            }
            TouchPhase::Moved => {
                let gesture_in_world = |points: &BTreeMap<u64, Vec2>| {
                    let mut pair = points.values();
                    let (first, second) = (*pair.next()?, *pair.next()?);
                    Some((first.distance(second).max(12.), (first + second) * 0.5))
                };
                let before = gesture_in_world(&controls.fingers);
                if let Some(pos) = controls.fingers.get_mut(&event.id) {
                    *pos = event.position;
                }
                if let (Some((before, previous)), Some((after, current))) =
                    (before, gesture_in_world(&controls.fingers))
                {
                    if play.panel == Panel::Closed && controls.pinch_in_world {
                        let scale = view.rendered_height / size.y.max(120.);
                        let delta = current - previous;
                        view.pan(-delta.x * scale, delta.y * scale);
                        view.zoom_toward(
                            (after / before).ln(),
                            ground_point(&game, camera, pose, previous),
                        );
                    }
                } else if let Some((finger, gesture)) = &mut controls.touch {
                    if *finger == event.id {
                        drag(gesture, event.position, &play, &mut view, size.y);
                    }
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                controls.fingers.remove(&event.id);
                if controls.fingers.len() < 2 {
                    controls.pinch_in_world = false;
                }
                if controls
                    .touch
                    .as_ref()
                    .is_some_and(|(id, _)| *id == event.id)
                {
                    let (_, gesture) = controls.touch.take().unwrap();
                    if event.phase == TouchPhase::Ended {
                        release(
                            gesture,
                            event.position,
                            &mut play,
                            &mut game,
                            &mut view,
                            camera,
                            pose,
                            &areas,
                            scale,
                            now,
                        );
                    }
                }
            }
        }
    }

    for event in mouse
        .read()
        .filter(|event| event.window == entity && event.button == MouseButton::Left)
    {
        if touched || !controls.fingers.is_empty() {
            continue;
        }
        play.gamepad = false;
        if let Some(cursor) = window.cursor_position() {
            play.pointer = Some(cursor);
            if event.state == ButtonState::Pressed {
                controls.mouse = Some(Drag::new(cursor, hud::hit(&areas, cursor, scale)));
            } else if let Some(gesture) = controls.mouse.take() {
                release(
                    gesture, cursor, &mut play, &mut game, &mut view, camera, pose, &areas, scale,
                    now,
                );
            }
        } else {
            controls.mouse = None;
        }
    }
    if let (Some(gesture), Some(cursor)) = (&mut controls.mouse, window.cursor_position()) {
        drag(gesture, cursor, &play, &mut view, size.y);
    }
    for scroll in wheel.read().filter(|scroll| scroll.window == entity) {
        if play.panel == Panel::Closed {
            view.zoom(
                scroll.y
                    * match scroll.unit {
                        MouseScrollUnit::Line => 0.105,
                        MouseScrollUnit::Pixel => 0.002,
                    },
            );
        }
    }
    let keyboard_axis =
        |positive, negative| i32::from(keys.pressed(positive)) - i32::from(keys.pressed(negative));
    if play.panel == Panel::Closed {
        view.pan(
            keyboard_axis(KeyCode::KeyD, KeyCode::KeyA) as f32 * dt * 12.,
            keyboard_axis(KeyCode::KeyW, KeyCode::KeyS) as f32 * dt * 12.,
        );
    }
    for (key, tool) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
    ]
    .into_iter()
    .zip(Tool::ALL)
    {
        if keys.just_pressed(key) && play.panel == Panel::Closed {
            play.activate(UiAction::Tool(tool), &mut game, &mut view, now);
        }
    }
    for (key, action) in [
        (KeyCode::KeyE, UiAction::Confirm),
        (KeyCode::KeyR, UiAction::Rotate),
        (KeyCode::KeyG, UiAction::Guide),
        (KeyCode::Escape, UiAction::Cancel),
        (KeyCode::Tab, UiAction::Panel(Panel::Journey)),
        (KeyCode::KeyM, UiAction::Panel(Panel::Map)),
        (KeyCode::KeyP, UiAction::Panel(Panel::Progress)),
        (KeyCode::KeyQ, UiAction::Panel(Panel::Quests)),
        (KeyCode::KeyB, UiAction::Panel(Panel::Catalog)),
        (KeyCode::KeyT, UiAction::Panel(Panel::Research)),
        (KeyCode::KeyI, UiAction::Panel(Panel::Inventory)),
        (KeyCode::KeyF, UiAction::Panel(Panel::Modes)),
        (KeyCode::F1, UiAction::Panel(Panel::Menu)),
    ] {
        if keys.just_pressed(key) {
            play.activate(action, &mut game, &mut view, now);
        }
    }
    if play.panel != Panel::Closed {
        let direction = Vec2::new(
            f32::from(keys.just_pressed(KeyCode::ArrowRight))
                - f32::from(keys.just_pressed(KeyCode::ArrowLeft)),
            f32::from(keys.just_pressed(KeyCode::ArrowDown))
                - f32::from(keys.just_pressed(KeyCode::ArrowUp)),
        );
        if direction != Vec2::ZERO || keys.just_pressed(KeyCode::Enter) {
            play.focus = hud::move_focus(&areas, play.focus, direction);
            if keys.just_pressed(KeyCode::Enter) {
                if let Some(action) = play.focus {
                    play.activate(action, &mut game, &mut view, now);
                }
            }
        }
    }

    let pad = read_pad(&gamepads);
    let pressed = if controls.resume_pad {
        0
    } else {
        (pad.buttons & !controls.pad_buttons) | pad.pressed
    };
    controls.resume_pad = false;
    controls.pad_buttons = pad.buttons;
    if !pad.connected {
        play.gamepad = false;
    }
    if !pad.connected || touched || !controls.fingers.is_empty() {
        return;
    }
    let left = deadzone(pad.left);
    let right = deadzone(pad.right);
    if pressed != 0 || left != Vec2::ZERO || right != Vec2::ZERO || pad.zoom.abs() > 0.13 {
        if !play.gamepad {
            let aim = play.aim.world();
            play.pointer = camera
                .world_to_viewport(
                    pose,
                    aim.with_y(terrain::actor_height(
                        game.region(),
                        aim,
                        game.city().bridge_fixed,
                    )),
                )
                .ok()
                .or(Some(size * 0.5));
            play.gamepad = true;
        }
    }
    if !play.gamepad {
        return;
    }
    if play.panel == Panel::Closed {
        view.pan(right.x * dt * 16., right.y * dt * 16.);
        if pad.zoom.abs() > 0.13 {
            view.zoom(pad.zoom * dt * 1.20);
        }
    }
    let horizontal = i32::from(pressed & RIGHT != 0) - i32::from(pressed & LEFT != 0);
    let vertical = i32::from(pressed & DOWN != 0) - i32::from(pressed & UP != 0);
    if play.panel == Panel::Closed {
        if horizontal != 0 {
            let i = (play.tool as i32 + horizontal).rem_euclid(Tool::ALL.len() as i32);
            play.tool(Tool::ALL[i as usize]);
        }
        let edge = Vec2::new(19., 25.);
        let cursor = (play.pointer.unwrap_or(size * 0.5)
            + Vec2::new(left.x, -left.y) * size.y * 0.83 * dt)
            .clamp(edge, (size - edge).max(edge));
        play.pointer = Some(cursor);
        let over_ui = hud::hit(&areas, cursor, scale).0;
        if !over_ui && (left != Vec2::ZERO || right != Vec2::ZERO || pad.zoom.abs() > 0.13) {
            select(&mut play, &game, camera, pose, cursor);
        }
        if pressed & SOUTH != 0 {
            play.activate(UiAction::Confirm, &mut game, &mut view, now);
        }
        if vertical != 0 && play.panel == Panel::Closed {
            let action = if vertical > 0 {
                UiAction::Guide
            } else if play.tool == Tool::Hand {
                UiAction::Panel(Panel::Menu)
            } else {
                UiAction::Tool(play.tool)
            };
            play.activate(action, &mut game, &mut view, now);
        }
    } else {
        let direction = Vec2::new(horizontal as f32, vertical as f32);
        play.focus = hud::move_focus(&areas, play.focus, direction);
        if pressed & SOUTH != 0 {
            if let Some(action) = play.focus {
                play.activate(action, &mut game, &mut view, now);
            }
        }
    }
    let context = if play.panel == Panel::Closed && play.tool != Tool::Hand {
        UiAction::Tool(play.tool)
    } else {
        UiAction::Panel(Panel::Progress)
    };
    for (bit, action) in [
        (EAST, UiAction::Cancel),
        (MENU, UiAction::Panel(Panel::Menu)),
        (ROTATE, UiAction::Rotate),
        (GUIDE, UiAction::Guide),
        (PROGRESS, context),
    ] {
        if pressed & bit != 0 {
            play.activate(action, &mut game, &mut view, now);
        }
    }
}
