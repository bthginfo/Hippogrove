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
    play::{CATALOG_PAGE_SIZE, CameraRig, Panel, Play, Tool, UiAction},
    region::{CELL, Material, Tile},
    terrain,
    view::{Board, WorldCamera},
    wheel::Wheel,
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
const WHEEL: u32 = 512;
const MAP: u32 = 1024;
const MIRACLES: u32 = 2048;
const OBJECTIVE: u32 = 4096;
const CONTEXT: u32 = 8192;

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
        let map = [
            (GamepadButton::South, SOUTH),
            (GamepadButton::East, EAST),
            (GamepadButton::DPadLeft, LEFT),
            (GamepadButton::DPadRight, RIGHT),
            (GamepadButton::DPadUp, UP),
            (GamepadButton::DPadDown, DOWN),
            (GamepadButton::Start, MENU),
            (GamepadButton::West, ROTATE),
            (GamepadButton::RightTrigger, GUIDE),
            (GamepadButton::LeftTrigger, WHEEL),
            (GamepadButton::Select, MAP),
            (GamepadButton::North, MIRACLES),
            (GamepadButton::LeftThumb, OBJECTIVE),
            (GamepadButton::RightThumb, CONTEXT),
        ];
        let axis = |axis| pad.get(axis).unwrap_or(0.);
        Pad {
            left: Vec2::new(axis(GamepadAxis::LeftStickX), axis(GamepadAxis::LeftStickY)),
            right: Vec2::new(
                axis(GamepadAxis::RightStickX),
                axis(GamepadAxis::RightStickY),
            ),
            zoom: pad.get(GamepadButton::RightTrigger2).unwrap_or(0.)
                - pad.get(GamepadButton::LeftTrigger2).unwrap_or(0.),
            buttons: map
                .iter()
                .filter(|&&(button, _)| pad.pressed(button))
                .map(|&(_, bit)| bit)
                .fold(0, |bits, bit| bits | bit),
            connected: true,
            pressed: map
                .iter()
                .filter(|&&(button, _)| pad.just_pressed(button))
                .map(|&(_, bit)| bit)
                .fold(0, |bits, bit| bits | bit),
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
    grab: bool,
}

impl Drag {
    fn new(
        position: Vec2,
        (over_ui, pressed): (bool, Option<UiAction>),
        play: &mut Play,
        game: &Game,
        camera: &Camera,
        pose: &GlobalTransform,
        now: f32,
    ) -> Self {
        let world_hand = !over_ui
            && play.panel == Panel::Closed
            && play.tool == Tool::Hand
            && select(play, game, camera, pose, position);
        Self {
            start: position,
            last: position,
            over_ui,
            pressed,
            dragging: false,
            grab: (world_hand || pressed == Some(UiAction::Confirm)) && play.start_grab(game, now),
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
    key_grab: bool,
    pad_grab: bool,
    menu_direction: MenuDirection,
    pad_wheel: Option<Wheel>,
    stick_waits_for_center: bool,
    mouse_position: Option<Vec2>,
}

#[derive(Default)]
struct MenuDirection {
    panel: Panel,
    active: Vec2,
    repeat_at: f32,
}

impl MenuDirection {
    fn update(&mut self, panel: Panel, buttons: u32, stick: Vec2, now: f32) -> Vec2 {
        let dpad = Vec2::new(
            f32::from(buttons & RIGHT != 0) - f32::from(buttons & LEFT != 0),
            f32::from(buttons & DOWN != 0) - f32::from(buttons & UP != 0),
        );
        let axes = if dpad != Vec2::ZERO {
            dpad
        } else {
            stick * Vec2::new(1., -1.)
        };
        let axis = axes.abs();
        let direction = if axis.max_element() < 0.49 {
            Vec2::ZERO
        } else if axis.x >= axis.y {
            Vec2::new(axes.x.signum(), 0.)
        } else {
            Vec2::new(0., axes.y.signum())
        };
        if panel != self.panel {
            self.panel = panel;
            self.active = direction;
            self.repeat_at = now + 0.36;
            return Vec2::ZERO;
        }
        if self.active != direction {
            self.active = direction;
            self.repeat_at = now + 0.36;
            direction
        } else if direction != Vec2::ZERO && now >= self.repeat_at {
            self.repeat_at = now + 0.13;
            direction
        } else {
            Vec2::ZERO
        }
    }
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

fn select(
    play: &mut Play,
    game: &Game,
    camera: &Camera,
    pose: &GlobalTransform,
    screen: Vec2,
) -> bool {
    let region = game.region();
    let city = game.city();
    let Some(point) = ground_point(game, camera, pose, screen) else {
        return false;
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
            if let Some(farm) = city.opening_farm(region) {
                consider(farm, farm.world() + Vec3::Y * 0.8);
            }
            if !region.tutorial {
                consider(region.bridge(), region.bridge().world() + Vec3::Y * 0.23);
                for district in &region.districts {
                    consider(district.center, district.center.world() + Vec3::Y * 0.55);
                }
            }
            if city.held.is_none() {
                let hippo = &city.walkers[0];
                consider(
                    hippo.tile(),
                    hippo.world() + Vec3::Y * (0.86 * game.campaign.gifts.creature.scale()),
                );
            }
            if city.held.is_none() {
                for walker in city.walkers.iter().filter(|w| !w.hippo && !w.guard) {
                    consider(walker.tile(), walker.world() + Vec3::Y * 0.66);
                }
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
            if city.rival_active(region) {
                let point = city.rival_world(region);
                consider(Tile::from_world(point), point + Vec3::Y * 1.2);
            }
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
        if tile != play.aim {
            play.grab = None;
        }
        play.aim = tile;
        return true;
    }
    false
}

fn drag(gesture: &mut Drag, position: Vec2, play: &mut Play, view: &mut CameraRig, height: f32) {
    let delta = position - gesture.last;
    if !gesture.dragging && gesture.start.distance(position) > 9. {
        gesture.dragging = true;
        if gesture.grab {
            play.grab = None;
        }
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
    if gesture.grab {
        if !gesture.dragging
            && gesture.start.distance(position) <= 15.
            && (action == Some(UiAction::Confirm) || (!gesture.over_ui && !over_ui))
        {
            if gesture.over_ui || play.grab.is_some_and(|hold| now - hold.started >= 0.36) {
                play.finish_grab(game, view, now);
            } else {
                play.grab = None;
                play.inspect_world(game, now);
            }
        } else {
            play.grab = None;
        }
        return;
    }
    if !gesture.dragging && gesture.start.distance(position) <= 15. {
        if gesture.over_ui {
            if let Some(action) = action.filter(|action| Some(*action) == gesture.pressed) {
                play.activate(action, game, view, now);
            }
        } else if !over_ui && play.panel == Panel::Context {
            play.activate(UiAction::Cancel, game, view, now);
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
                    .world_to_viewport(
                        pose,
                        hippo.with_y(ground + 0.76 * game.campaign.gifts.creature.scale()),
                    )
                    .is_ok_and(|tip| tip.distance(position) <= 19.)
            {
                play.aim = city.walkers[0].tile();
                play.inspect_world(game, now);
                return;
            }
            if select(play, game, camera, pose, position) {
                play.inspect_world(game, now);
            }
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
        play.grab = None;
        controls.fingers.clear();
        controls.touch = None;
        controls.mouse = None;
        controls.pinch_in_world = false;
        controls.resume_pad = true;
        controls.key_grab = false;
        controls.pad_grab = false;
        controls.menu_direction = MenuDirection::default();
        controls.pad_wheel = None;
        controls.stick_waits_for_center = false;
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
                    controls.touch = Some((
                        event.id,
                        Drag::new(event.position, hit, &mut play, &game, camera, pose, now),
                    ));
                } else {
                    if controls.fingers.len() == 2 {
                        controls.pinch_in_world = !hit.0
                            && controls
                                .touch
                                .as_ref()
                                .is_some_and(|(_, first)| !first.over_ui);
                    }
                    if controls
                        .touch
                        .take()
                        .is_some_and(|(_, gesture)| gesture.grab)
                    {
                        play.grab = None;
                    }
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
                        drag(gesture, event.position, &mut play, &mut view, size.y);
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
                    } else if gesture.grab {
                        play.grab = None;
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
                controls.mouse = Some(Drag::new(
                    cursor,
                    hud::hit(&areas, cursor, scale),
                    &mut play,
                    &game,
                    camera,
                    pose,
                    now,
                ));
            } else if let Some(gesture) = controls.mouse.take() {
                release(
                    gesture, cursor, &mut play, &mut game, &mut view, camera, pose, &areas, scale,
                    now,
                );
            }
        } else {
            if controls.mouse.take().is_some_and(|gesture| gesture.grab) {
                play.grab = None;
            }
        }
    }
    let mouse_position = window.cursor_position();
    let mouse_moved = controls.mouse_position != mouse_position;
    controls.mouse_position = mouse_position;
    if let (Some(gesture), Some(cursor)) = (&mut controls.mouse, mouse_position) {
        drag(gesture, cursor, &mut play, &mut view, size.y);
    }
    if mouse_moved
        && !touched
        && controls.fingers.is_empty()
        && controls.mouse.is_none()
        && play.panel == Panel::Closed
        && play.tool == Tool::Build
    {
        if let Some(cursor) = mouse_position.filter(|&cursor| !hud::hit(&areas, cursor, scale).0) {
            play.gamepad = false;
            play.pointer = Some(cursor);
            select(&mut play, &game, camera, pose, cursor);
        }
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
    if keys.just_pressed(KeyCode::KeyE) {
        controls.key_grab = play.start_grab(&game, now);
        if !controls.key_grab {
            play.activate(UiAction::Confirm, &mut game, &mut view, now);
        }
    }
    if keys.just_released(KeyCode::KeyE) && std::mem::take(&mut controls.key_grab) {
        play.finish_grab(&mut game, &mut view, now);
    }
    for (key, action) in [
        (KeyCode::KeyR, UiAction::Rotate),
        (KeyCode::KeyX, UiAction::Inspect),
        (KeyCode::KeyG, UiAction::Guide),
        (KeyCode::KeyH, UiAction::ToolTarget),
        (KeyCode::Equal, UiAction::Zoom(true)),
        (KeyCode::Minus, UiAction::Zoom(false)),
        (KeyCode::Escape, UiAction::Cancel),
        (KeyCode::Tab, UiAction::Panel(Panel::Journey)),
        (KeyCode::KeyM, UiAction::Panel(Panel::Map)),
        (KeyCode::KeyP, UiAction::Panel(Panel::Progress)),
        (KeyCode::KeyQ, UiAction::Panel(Panel::Quests)),
        (KeyCode::KeyB, UiAction::Panel(Panel::Catalog)),
        (KeyCode::KeyT, UiAction::Panel(Panel::Research)),
        (KeyCode::KeyI, UiAction::Panel(Panel::Inventory)),
        (KeyCode::KeyF, UiAction::MainMenu),
        (KeyCode::F1, UiAction::Panel(Panel::Menu)),
        (KeyCode::Space, UiAction::ToggleWheel),
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
            if let Panel::Wheel(wheel) = play.panel {
                let turn = (direction.x + direction.y).signum() as i32;
                let choice = wheel.step(play.focus, if direction == Vec2::ZERO { 0 } else { turn });
                play.set_focus(Some(choice));
            } else {
                let focus = hud::move_focus(&areas, play.focus, direction);
                play.set_focus(focus);
            }
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
        if std::mem::take(&mut controls.pad_grab) {
            play.grab = None;
        }
    }
    if !pad.connected || touched || !controls.fingers.is_empty() {
        return;
    }
    let raw_left = deadzone(pad.left);
    let right = deadzone(pad.right);
    if pressed != 0 || raw_left != Vec2::ZERO || right != Vec2::ZERO || pad.zoom.abs() > 0.13 {
        if !play.gamepad {
            play.pointer = None;
            play.gamepad = true;
        }
    }
    if !play.gamepad {
        return;
    }
    if play.panel == Panel::Closed && pressed & CONTEXT != 0 {
        play.inspect_world(&game, now);
        return;
    }
    if play.panel == Panel::Closed && pressed & OBJECTIVE != 0 {
        play.activate(UiAction::Guide, &mut game, &mut view, now);
        return;
    }
    if pressed & WHEEL != 0 && matches!(play.panel, Panel::Closed | Panel::Wheel(_)) {
        controls.pad_grab = false;
        play.activate(UiAction::ToggleWheel, &mut game, &mut view, now);
        return;
    }
    if matches!(play.panel, Panel::Closed | Panel::Wheel(_)) {
        let building = play.panel == Panel::Closed && play.tool == Tool::Build;
        let quick_wheel = if pressed & ROTATE != 0 && !building {
            Some(Wheel::Build)
        } else if pressed & MIRACLES != 0 {
            Some(Wheel::Miracles)
        } else if pressed & GUIDE != 0
            && (play.tool == Tool::Hand || matches!(play.panel, Panel::Wheel(_)))
        {
            Some(Wheel::Hippo)
        } else {
            None
        };
        if let Some(wheel) = quick_wheel {
            controls.pad_grab = false;
            play.activate(UiAction::QuickWheel(wheel), &mut game, &mut view, now);
            return;
        }
    }
    let wheel = match play.panel {
        Panel::Wheel(wheel) => Some(wheel),
        _ => None,
    };
    let centered = pad.left.length_squared() < 0.24 * 0.24;
    if controls.pad_wheel != wheel {
        controls.pad_wheel = wheel;
        controls.stick_waits_for_center = !centered;
    } else if centered {
        controls.stick_waits_for_center = false;
    }
    let left = if controls.stick_waits_for_center {
        Vec2::ZERO
    } else {
        raw_left
    };
    let horizontal = i32::from(pressed & RIGHT != 0) - i32::from(pressed & LEFT != 0);
    let vertical = i32::from(pressed & DOWN != 0) - i32::from(pressed & UP != 0);
    if let Some(wheel) = wheel {
        controls.menu_direction = MenuDirection::default();
        let turn = (horizontal + vertical).signum();
        let choice = if turn != 0 {
            wheel.step(play.focus, turn)
        } else if controls.stick_waits_for_center {
            wheel.step(play.focus, 0)
        } else {
            wheel.aim(play.focus, pad.left)
        };
        play.set_focus(Some(choice));
        let chosen = if pressed & EAST != 0 {
            Some(UiAction::Cancel)
        } else if pressed & MENU != 0 {
            Some(UiAction::Panel(Panel::Menu))
        } else if pressed & MAP != 0 {
            Some(UiAction::Panel(Panel::Map))
        } else if pressed & SOUTH != 0 {
            play.focus
        } else {
            None
        };
        if let Some(action) = chosen {
            play.activate(action, &mut game, &mut view, now);
        }
        return;
    }
    if play.panel == Panel::Catalog && pressed & (WHEEL | GUIDE) != 0 {
        let page = if pressed & WHEEL != 0 {
            play.catalog_page.saturating_sub(1)
        } else {
            let pages = play
                .catalog_entries(&game)
                .len()
                .div_ceil(CATALOG_PAGE_SIZE);
            (play.catalog_page + 1).min(pages.saturating_sub(1))
        };
        if page != play.catalog_page {
            play.activate(UiAction::CatalogPage(page), &mut game, &mut view, now);
        }
        return;
    }
    let building = play.panel == Panel::Closed && play.tool == Tool::Build;
    let zooming = pad.zoom.abs() > 0.13;
    if play.panel == Panel::Closed {
        view.pan(right.x * dt * 16., right.y * dt * 16.);
        if zooming {
            view.zoom_toward(pad.zoom * dt * 1.20, building.then(|| play.aim.world()));
        }
        if building {
            let nudge = controls.menu_direction.update(
                Panel::Closed,
                pad.buttons | pressed,
                Vec2::ZERO,
                now,
            );
            let bounds = game.region().bounds;
            let aim = play.aim.offset(nudge.x as i16, nudge.y as i16);
            if nudge != Vec2::ZERO && bounds.contains(aim) {
                play.aim = aim;
                play.guide_marker = None;
                play.pointer = None;
            } else if right != Vec2::ZERO || zooming {
                play.pointer = None;
            }
        } else {
            controls.menu_direction = MenuDirection::default();
            if horizontal != 0 {
                let i = (play.tool as i32 + horizontal).rem_euclid(Tool::ALL.len() as i32);
                play.tool(Tool::ALL[i as usize]);
            }
        }
        if left != Vec2::ZERO || (!building && (right != Vec2::ZERO || zooming)) {
            let position = play
                .pointer
                .or_else(|| aim_on_screen(&play, &game, camera, pose))
                .unwrap_or(size * 0.5);
            let edge = Vec2::new(19., 25.);
            let cursor = (position + Vec2::new(left.x, -left.y) * size.y * 0.83 * dt)
                .clamp(edge, (size - edge).max(edge));
            play.pointer = Some(cursor);
            if !hud::hit(&areas, cursor, scale).0 {
                select(&mut play, &game, camera, pose, cursor);
            }
        }
        if pressed & SOUTH != 0 {
            controls.pad_grab = play.start_grab(&game, now);
            if !controls.pad_grab {
                play.activate(UiAction::Confirm, &mut game, &mut view, now);
            }
        }
        if !building && vertical != 0 && play.panel == Panel::Closed {
            let action = if vertical < 0 {
                UiAction::Guide
            } else if play.tool == Tool::Hand {
                UiAction::Panel(Panel::Menu)
            } else {
                UiAction::Tool(play.tool)
            };
            play.activate(action, &mut game, &mut view, now);
        }
        if pressed & ROTATE != 0 && building {
            play.activate(UiAction::Rotate, &mut game, &mut view, now);
        }
        if pressed & GUIDE != 0 && play.panel == Panel::Closed {
            play.activate(UiAction::ToolTarget, &mut game, &mut view, now);
        }
    } else {
        let menu_stick = if controls.stick_waits_for_center {
            Vec2::ZERO
        } else {
            pad.left
        };
        let direction =
            controls
                .menu_direction
                .update(play.panel, pad.buttons | pressed, menu_stick, now);
        let focus = hud::move_focus(&areas, play.focus, direction);
        play.set_focus(focus);
        if pressed & SOUTH != 0 {
            if let Some(action) = play.focus {
                play.activate(action, &mut game, &mut view, now);
            }
        }
    }
    if pad.buttons & SOUTH == 0 && std::mem::take(&mut controls.pad_grab) {
        play.finish_grab(&mut game, &mut view, now);
    }
    for (bit, action) in [
        (EAST, UiAction::Cancel),
        (MENU, UiAction::Panel(Panel::Menu)),
        (MAP, UiAction::Panel(Panel::Map)),
    ] {
        if pressed & bit != 0 {
            play.activate(action, &mut game, &mut view, now);
        }
    }
}

pub fn aim_on_screen(
    play: &Play,
    game: &Game,
    camera: &Camera,
    pose: &GlobalTransform,
) -> Option<Vec2> {
    let aim = play.aim.world();
    camera
        .world_to_viewport(
            pose,
            aim.with_y(terrain::actor_height(
                game.region(),
                aim,
                game.city().bridge_fixed,
            )),
        )
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_navigation_supports_held_directions_without_carrying_a_move_into_the_next_panel() {
        let mut navigation = MenuDirection::default();
        let step = |navigation: &mut MenuDirection, panel, buttons, stick, time| {
            navigation.update(panel, buttons, stick, time)
        };
        assert_eq!(
            step(&mut navigation, Panel::Menu, RIGHT, Vec2::ZERO, 0.),
            Vec2::ZERO
        );
        assert_eq!(
            step(&mut navigation, Panel::Menu, 0, Vec2::new(0.17, 0.26), 0.1),
            Vec2::ZERO
        );
        assert_eq!(
            step(&mut navigation, Panel::Menu, 0, Vec2::NEG_Y, 0.2),
            Vec2::Y
        );
        assert_eq!(
            step(&mut navigation, Panel::Menu, 0, Vec2::NEG_Y, 0.3),
            Vec2::ZERO
        );
        assert_eq!(
            step(&mut navigation, Panel::Menu, 0, Vec2::NEG_Y, 0.60),
            Vec2::Y
        );
        assert_eq!(
            step(&mut navigation, Panel::Research, 0, Vec2::NEG_Y, 0.65),
            Vec2::ZERO
        );
        assert_eq!(
            step(&mut navigation, Panel::Research, LEFT, Vec2::NEG_Y, 0.7),
            Vec2::NEG_X
        );
    }
}
