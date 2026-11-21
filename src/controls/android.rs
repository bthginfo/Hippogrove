use std::{ffi::c_void, sync::Mutex};

use bevy::prelude::Vec2;
use ndk_sys::*;

use super::{
    CONTEXT, DOWN, EAST, GUIDE, LEFT, MAP, MENU, MIRACLES, OBJECTIVE, RIGHT, ROTATE, SOUTH, UP,
    WHEEL, native_pad,
};

struct Controller {
    id: Option<i32>,
    keys: u32,
    hat: u32,
    left: Vec2,
    right: Vec2,
    zoom: f32,
}

impl Controller {
    const EMPTY: Self = Self {
        id: None,
        keys: 0,
        hat: 0,
        left: Vec2::ZERO,
        right: Vec2::ZERO,
        zoom: 0.,
    };

    fn publish(&self) {
        native_pad(
            self.left.x,
            self.left.y,
            self.right.x,
            self.right.y,
            self.zoom,
            self.keys | self.hat,
            self.id.is_some(),
        );
    }
}

static CONTROLLER: Mutex<Controller> = Mutex::new(Controller::EMPTY);

fn key_bit(code: u32) -> Option<u32> {
    Some(match code {
        AKEYCODE_BUTTON_A => SOUTH,
        AKEYCODE_BUTTON_B => EAST,
        AKEYCODE_DPAD_LEFT => LEFT,
        AKEYCODE_DPAD_RIGHT => RIGHT,
        AKEYCODE_DPAD_UP => UP,
        AKEYCODE_DPAD_DOWN => DOWN,
        AKEYCODE_BUTTON_START => MENU,
        AKEYCODE_BUTTON_SELECT => MAP,
        AKEYCODE_BUTTON_X => ROTATE,
        AKEYCODE_BUTTON_Y => MIRACLES,
        AKEYCODE_BUTTON_THUMBL => OBJECTIVE,
        AKEYCODE_BUTTON_THUMBR => CONTEXT,
        AKEYCODE_BUTTON_R1 => GUIDE,
        AKEYCODE_BUTTON_L1 => WHEEL,
        _ => return None,
    })
}

fn controller_event(event: *const AInputEvent) -> bool {
    // The native queue owns this event until it is finished by the caller.
    let (source, kind, device) = unsafe {
        (
            AInputEvent_getSource(event) as u32,
            AInputEvent_getType(event) as u32,
            AInputEvent_getDeviceId(event),
        )
    };
    let has_source = |candidate| source & candidate == candidate;
    let is_key = kind == AINPUT_EVENT_TYPE_KEY
        && (has_source(AINPUT_SOURCE_GAMEPAD) || has_source(AINPUT_SOURCE_DPAD));
    let is_stick = kind == AINPUT_EVENT_TYPE_MOTION && has_source(AINPUT_SOURCE_JOYSTICK);
    if !is_key && !is_stick {
        return false;
    }
    let key = if is_key {
        let Some(bit) = key_bit(unsafe { AKeyEvent_getKeyCode(event) } as u32) else {
            return false;
        };
        Some((
            bit,
            unsafe { AKeyEvent_getAction(event) } as u32 == AKEY_EVENT_ACTION_DOWN,
        ))
    } else {
        None
    };
    let mut state = CONTROLLER.lock().unwrap_or_else(|state| state.into_inner());
    if state.id != Some(device) {
        *state = Controller {
            id: Some(device),
            ..Controller::EMPTY
        };
    }
    if let Some((bit, down)) = key {
        if down {
            state.keys |= bit;
        } else {
            state.keys &= !bit;
        }
    } else {
        let axis =
            |axis| unsafe { AMotionEvent_getAxisValue(event, axis as i32, 0) }.clamp(-1., 1.);
        let right_axis = |primary, fallback| {
            let preferred = axis(primary);
            if preferred.abs() > 0.05 {
                preferred
            } else {
                axis(fallback)
            }
        };
        state.left = Vec2::new(axis(AMOTION_EVENT_AXIS_X), -axis(AMOTION_EVENT_AXIS_Y));
        state.right = Vec2::new(
            right_axis(AMOTION_EVENT_AXIS_Z, AMOTION_EVENT_AXIS_RX),
            -right_axis(AMOTION_EVENT_AXIS_RZ, AMOTION_EVENT_AXIS_RY),
        );
        let right = axis(AMOTION_EVENT_AXIS_RTRIGGER).max(axis(AMOTION_EVENT_AXIS_GAS));
        let left = axis(AMOTION_EVENT_AXIS_LTRIGGER).max(axis(AMOTION_EVENT_AXIS_BRAKE));
        state.zoom = right - left;
        let x = axis(AMOTION_EVENT_AXIS_HAT_X);
        let y = axis(AMOTION_EVENT_AXIS_HAT_Y);
        state.hat = if x < -0.5 {
            LEFT
        } else if x > 0.5 {
            RIGHT
        } else {
            0
        } | if y < -0.5 {
            UP
        } else if y > 0.5 {
            DOWN
        } else {
            0
        };
    }
    state.publish();
    true
}

unsafe extern "C" {
    fn __real_AInputQueue_getEvent(queue: *mut AInputQueue, event: *mut *mut AInputEvent) -> i32;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn __wrap_AInputQueue_getEvent(
    queue: *mut AInputQueue,
    event: *mut *mut AInputEvent,
) -> i32 {
    loop {
        let result = unsafe { __real_AInputQueue_getEvent(queue, event) };
        if result < 0 || !controller_event(unsafe { *event }) {
            return result;
        }
        // Winit receives the rest of the queue unchanged and finishes those events itself.
        unsafe { AInputQueue_finishEvent(queue, *event, 1) };
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_game_hippogrove_GameActivity_attachController(
    _env: *mut c_void,
    _class: *mut c_void,
    id: i32,
) {
    let mut state = CONTROLLER.lock().unwrap_or_else(|state| state.into_inner());
    if state.id.is_none() {
        state.id = Some(id);
        state.publish();
    }
}

#[unsafe(no_mangle)]
extern "system" fn Java_game_hippogrove_GameActivity_resetController(
    _env: *mut c_void,
    _class: *mut c_void,
    id: i32,
) {
    let mut state = CONTROLLER.lock().unwrap_or_else(|state| state.into_inner());
    if id < 0 || state.id == Some(id) {
        *state = Controller::EMPTY;
        state.publish();
    }
}
