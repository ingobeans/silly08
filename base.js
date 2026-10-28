import { send_data, timeout } from "ext:core/ops";
import "fake-indexeddb:auto";

function constructProxy(name, items, log = false, onSet = undefined) {
    if (!log) {
        return items;
    }
    return new Proxy(items, {
        get(target, property) {
            console.log("[PROXY]: " + name + " : " + property);
            return property in target ? target[property] : undefined;
        },
        set(target, property, value) {
            console.log("[PROXY SET]: " + name + " : " + property);
            if (onSet) {
                onSet(property, value);
            }
            target[property] = value;
            return true;
        }
    })
}

var Location = constructProxy("location", {
    pathname: "silly08"
});

var ImageData = constructProxy("ImageData", {
    data: new Uint8Array(128 * 128 * 4)
});

var context = constructProxy("context", {
    createImageData: (w, h) => {
        return ImageData;
    },
    putImageData: (imageData, dx, dy, dirtyX, dirtyY, dirtyWidth, dirtyHeight) => {
        let a = new Uint8Array(imageData.data.buffer, imageData.data.byteOffset, imageData.data.length);
        send_data(a);
    }
});

var canvas = constructProxy("canvas", {
    addEventListener: addEventListener,
    getContext: () => { return context },
});

var Module = constructProxy("Module", {
    canvas: canvas,
    onAbort: (what) => { console.log("ABORT: " + what) }
});
var activeElement = constructProxy("activeElement", {
    tagName: "html",
});
var document = constructProxy("document", {
    addEventListener: addEventListener,
    getElementById: (id) => {
        return null;
    },
    activeElement: activeElement,
    currentScript: null,
});
var window = constructProxy("window", {
    addEventListener: addEventListener,
    location: Location,
});

var KeyboardEvent = constructProxy("KeyboardEvent", {
    DOM_KEY_LOCATION_RIGHT: 2
});

var tempDouble, tempI64, el, codo_key_buffer, codo_mounted_filesystem, temp, codo_wheel_dx, codo_wheel_dy, button_i = undefined;

let keyEventListeners = {
    keydown: [],
    keyup: [],
}

function addEventListener(event, callback) {
    if (event in keyEventListeners) {
        keyEventListeners[event].push(callback);
    }
}
function toKeyboardEvent(char, shift) {
    let keyCode = char == "Escape" ? 27 : undefined;
    return constructProxy("char", {
        key: char,
        code: char,
        type: "keydown",
        keyCode: keyCode,
    });
}

function sendEvents(targets, keys) {
    let pressed = keys.map(toKeyboardEvent);
    for (let ev of pressed) {
        for (let item of targets) {
            item(ev)
        }
    }
}

globalThis.setTimeout = function (callback, time) {
    timeout(time).then(([pressedKeys, heldKeys]) => {
        if (time != 0) {
            if (pressedKeys.length > 0) {
                // console.log("Keys: " + pressedKeys);
            }
            sendEvents(keyEventListeners.keydown, pressedKeys);
            // sendEvents(keyEventListeners.keydown,heldKeys);
        }
        callback();
    });
}
