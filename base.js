import { send_data, timeout, serialize, deserialize } from "ext:core/ops";
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
        if (id == "p8_file_chooser") {
            return {
                click: () => {

                    p8_dropped_cart = sample;
                    p8_dropped_cart_name = "wah.p8";
                    codo_command = 9;
                    // p8_drop_file(constructProxy("FILE OBJECT", {}, true))
                }
            }
        }
        return null;
    },
    activeElement: activeElement,
    currentScript: null,
});

class XMLHttpRequest {
    open() { }
    send() { this.onload() }
    status = 200;
    response = null;
}

var window = constructProxy("window", {
    addEventListener: addEventListener,
    location: Location,
});

var KeyboardEvent = constructProxy("KeyboardEvent", {
    DOM_KEY_LOCATION_RIGHT: 2
});

function download_browser_file(...a) {
    console.log("SAVE: " + a);
}

var tempDouble, tempI64, jsString, codo_command, p8_dropped_cart, p8_dropped_cart_name, _dir_name, el, codo_key_buffer, codo_mounted_filesystem, temp, codo_wheel_dx, codo_wheel_dy, button_i = undefined;


p8_dropped_cart = sample;
p8_dropped_cart_name = "wah.p8";
codo_command = 9;

let keyEventListeners = {
    keydown: [],
    keyup: [],
}

function addEventListener(event, callback) {
    if (event in keyEventListeners) {
        keyEventListeners[event].push(callback);
    }
}

let shift = false;
function toKeyboardEvent(char, type) {
    let keyCode = undefined;

    if (char == "Space") {
        char = " ";
    }

    if (char == "Escape") {
        keyCode = 27;
    } else if (char == "Enter") {
        keyCode = 13;
    }

    let dirIndex = ["Left", "Up", "Right", "Down"].indexOf(char)
    if (dirIndex != -1) {
        char = "Arrow" + char;
        keyCode = 37 + dirIndex;
    }

    if (char.startsWith("Key")) {
        char = char.substring(3);
    }
    if (char.length == 1) {
        keyCode = char.charCodeAt(0);
        char = char.toLowerCase();
    }

    // todo: replace hardcoded swedish keyboard layout
    // (teporary fix)
    let asInt = parseInt(char);
    if (asInt || asInt == 0) {
        if (shift) {
            char = "=!\"#¤%&/()"[asInt];
        }
    }

    return constructProxy("char", {
        key: char,
        code: char,
        type: type,
        keyCode: keyCode,
    });
}

function sendEvents(targets, keys, type) {
    let pressed = keys.map((e) => { return toKeyboardEvent(e, type) });
    for (let ev of pressed) {
        for (let item of targets) {
            item(ev)
        }
    }
}

// https://github.com/SollyBunny/structuredClone
globalThis.structuredClone = function (source, options) {
    options = options || {};
    options.transfer = options.transfer || [];
    let value = source;
    if (Array.isArray(source)) {
        if (options.transfer.indexOf(source) === -1) {
            options.transfer.push(source);
            value = [...source];
            for (let i = 0; i < value.length; ++i)
                value[i] = structuredClone(value[name]);
        }
    } else if (source === null) {
        value = null;
    } else if (typeof source === "object" && source.__proto__ === Object.prototype) {
        if (options.transfer.indexOf(source) === -1) {
            options.transfer.push(source);
            value = Object.assign({}, source);
            for (const name in value)
                value[name] = structuredClone(value[name]);
        }
    }
    return value;
}

let heldKeys = {}

globalThis.setTimeout = function (callback, time) {
    timeout(time).then(([keysAvailable, pressedKeys, releasedKeys]) => {
        if (keysAvailable) {
            // track map of pressed keys for debug purposes
            for (let k of pressedKeys) {
                heldKeys[k] = true;
            }
            for (let k of releasedKeys) {
                heldKeys[k] = false;
            }
            shift = heldKeys["LeftShift"] == true;

            sendEvents(keyEventListeners.keydown, pressedKeys, "keydown");
            sendEvents(keyEventListeners.keyup, releasedKeys, "keyup");
            if (pressedKeys.includes("RightShift")) {
                console.log(codo_command);
            }
        }
        callback();
    });
}

// // let s = serialize({ b: 2, a: () => { } }); // fails
// let s = serialize("{ b: 2 }");
// console.log(typeof s);
// let g = deserialize(new Uint8Array(s));
// console.log(g);
// while (true) { }