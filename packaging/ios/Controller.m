#import <Foundation/Foundation.h>
#import <GameController/GameController.h>

#include <stdint.h>

extern void hippogrove_controller_state(float lx, float ly, float rx, float ry,
                                        float zoom, uint32_t buttons, uint32_t connected);

static GCController *active;

static void publish(GCExtendedGamepad *pad) {
    uint32_t buttons = (pad.buttonA.pressed ? 1 : 0)
        | (pad.buttonB.pressed ? 2 : 0)
        | (pad.dpad.left.pressed ? 4 : 0)
        | (pad.dpad.right.pressed ? 8 : 0)
        | (pad.dpad.up.pressed ? 16 : 0)
        | (pad.dpad.down.pressed ? 32 : 0)
        | (pad.buttonMenu.pressed ? 64 : 0)
        | (pad.buttonX.pressed ? 128 : 0)
        | (pad.rightShoulder.pressed ? 256 : 0)
        | (pad.leftShoulder.pressed ? 512 : 0);
    hippogrove_controller_state(pad.leftThumbstick.xAxis.value, pad.leftThumbstick.yAxis.value,
        pad.rightThumbstick.xAxis.value, pad.rightThumbstick.yAxis.value,
        pad.rightTrigger.value - pad.leftTrigger.value, buttons, 1);
}

static void select_controller(GCController *controller) {
    if (!controller.extendedGamepad) return;
    active.extendedGamepad.valueChangedHandler = nil;
    active = controller;
    controller.handlerQueue = dispatch_get_main_queue();
    controller.extendedGamepad.valueChangedHandler = ^(GCExtendedGamepad *pad, GCControllerElement *element) {
        (void)element;
        publish(pad);
    };
    publish(controller.extendedGamepad);
}

void hippogrove_start_controllers(void) {
    NSNotificationCenter *center = NSNotificationCenter.defaultCenter;
    [center addObserverForName:GCControllerDidConnectNotification object:nil
                        queue:NSOperationQueue.mainQueue usingBlock:^(NSNotification *note) {
        select_controller(note.object);
    }];
    [center addObserverForName:GCControllerDidDisconnectNotification object:nil
                        queue:NSOperationQueue.mainQueue usingBlock:^(NSNotification *note) {
        if (note.object != active) return;
        active.extendedGamepad.valueChangedHandler = nil;
        active = nil;
        hippogrove_controller_state(0, 0, 0, 0, 0, 0, 0);
        for (GCController *controller in GCController.controllers) {
            if (controller != note.object && controller.extendedGamepad) {
                select_controller(controller);
                break;
            }
        }
    }];
    for (GCController *controller in GCController.controllers) {
        if (controller.extendedGamepad) {
            select_controller(controller);
            break;
        }
    }
}
