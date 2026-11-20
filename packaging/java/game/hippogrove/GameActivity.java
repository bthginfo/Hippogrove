package game.hippogrove;

import android.app.NativeActivity;
import android.content.Context;
import android.hardware.input.InputManager;
import android.os.Build;
import android.view.InputDevice;
import android.view.KeyEvent;
import android.view.MotionEvent;
import android.view.View;
import android.view.WindowInsets;
import android.view.WindowInsetsController;
import android.view.WindowManager;

public final class GameActivity extends NativeActivity implements InputManager.InputDeviceListener {
    static { System.loadLibrary("hippogrove"); }

    private static native void publishController(float lx, float ly, float rx, float ry,
            float zoom, int buttons, boolean connected);

    private InputManager inputs;
    private int device = -1;
    private int keys = 0;
    private int hat = 0;
    private float lx, ly, rx, ry, zoom;

    @Override
    protected void onCreate(android.os.Bundle state) {
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        super.onCreate(state);
        inputs = (InputManager) getSystemService(Context.INPUT_SERVICE);
    }

    @Override
    protected void onResume() {
        super.onResume();
        inputs.registerInputDeviceListener(this, null);
    }

    @Override
    protected void onPause() {
        inputs.unregisterInputDeviceListener(this);
        disconnect();
        super.onPause();
    }

    @Override
    public void onWindowFocusChanged(boolean focused) {
        super.onWindowFocusChanged(focused);
        if (!focused) {
            disconnect();
            return;
        }
        if (Build.VERSION.SDK_INT >= 30) {
            WindowInsetsController controller = getWindow().getInsetsController();
            if (controller != null) {
                controller.setSystemBarsBehavior(WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
                controller.hide(WindowInsets.Type.systemBars());
            }
        } else {
            getWindow().getDecorView().setSystemUiVisibility(View.SYSTEM_UI_FLAG_FULLSCREEN
                    | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION | View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY);
        }
    }

    private void disconnect() {
        device = -1;
        keys = hat = 0;
        lx = ly = rx = ry = zoom = 0;
        publishController(0, 0, 0, 0, 0, 0, false);
    }

    private void select(int id) {
        if (device != id) {
            disconnect();
            device = id;
        }
    }

    private void publish() {
        publishController(lx, ly, rx, ry, zoom, keys | hat, device != -1);
    }

    @Override
    public boolean dispatchKeyEvent(KeyEvent event) {
        if (event.isFromSource(InputDevice.SOURCE_GAMEPAD) || event.isFromSource(InputDevice.SOURCE_DPAD)) {
            int bit = keyBit(event.getKeyCode());
            if (bit != 0) {
                select(event.getDeviceId());
                if (event.getAction() == KeyEvent.ACTION_DOWN) {
                    keys |= bit;
                } else if (event.getAction() == KeyEvent.ACTION_UP) {
                    keys &= ~bit;
                }
                publish();
                return true;
            }
        }
        return super.dispatchKeyEvent(event);
    }

    @Override
    public boolean dispatchGenericMotionEvent(MotionEvent event) {
        if (!event.isFromSource(InputDevice.SOURCE_JOYSTICK) || event.getAction() != MotionEvent.ACTION_MOVE) {
            return super.dispatchGenericMotionEvent(event);
        }
        select(event.getDeviceId());
        lx = axis(event, MotionEvent.AXIS_X);
        ly = -axis(event, MotionEvent.AXIS_Y);
        boolean hasZ = hasAxis(event, MotionEvent.AXIS_Z);
        rx = axis(event, hasZ ? MotionEvent.AXIS_Z : MotionEvent.AXIS_RX);
        ry = -axis(event, hasZ ? MotionEvent.AXIS_RZ : MotionEvent.AXIS_RY);
        zoom = Math.max(axis(event, MotionEvent.AXIS_RTRIGGER), axis(event, MotionEvent.AXIS_GAS))
                - Math.max(axis(event, MotionEvent.AXIS_LTRIGGER), axis(event, MotionEvent.AXIS_BRAKE));
        float hx = axis(event, MotionEvent.AXIS_HAT_X);
        float hy = axis(event, MotionEvent.AXIS_HAT_Y);
        hat = (hx < -0.5f ? 4 : hx > 0.5f ? 8 : 0)
                | (hy < -0.5f ? 16 : hy > 0.5f ? 32 : 0);
        publish();
        return true;
    }

    private static boolean hasAxis(MotionEvent event, int axis) {
        return event.getDevice() != null && event.getDevice().getMotionRange(axis, event.getSource()) != null;
    }

    private static float axis(MotionEvent event, int axis) {
        if (!hasAxis(event, axis)) return 0;
        InputDevice.MotionRange range = event.getDevice().getMotionRange(axis, event.getSource());
        float value = event.getAxisValue(axis);
        return Math.abs(value) > range.getFlat() ? Math.max(-1f, Math.min(1f, value)) : 0f;
    }

    private static int keyBit(int key) {
        switch (key) {
            case KeyEvent.KEYCODE_BUTTON_A: return 1;
            case KeyEvent.KEYCODE_BUTTON_B: return 2;
            case KeyEvent.KEYCODE_DPAD_LEFT: return 4;
            case KeyEvent.KEYCODE_DPAD_RIGHT: return 8;
            case KeyEvent.KEYCODE_DPAD_UP: return 16;
            case KeyEvent.KEYCODE_DPAD_DOWN: return 32;
            case KeyEvent.KEYCODE_BUTTON_START: return 64;
            case KeyEvent.KEYCODE_BUTTON_X: return 128;
            case KeyEvent.KEYCODE_BUTTON_R1: return 256;
            case KeyEvent.KEYCODE_BUTTON_L1: return 512;
            default: return 0;
        }
    }

    @Override public void onInputDeviceAdded(int id) { }
    @Override public void onInputDeviceChanged(int id) { }
    @Override public void onInputDeviceRemoved(int id) { if (device == id) disconnect(); }
}
