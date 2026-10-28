import { test, send_data, timeout } from "ext:core/ops";

function constructProxy(name, items) {
    return new Proxy(items, {
        get(target, property) {
            console.log("[PROXY]: " + name + " : " + property);
            return property in target ? target[property] : undefined;
        }
    })
}

var DBOpenRequest = constructProxy("DBOpenRequest", {

});

var Location = constructProxy("location", {
    pathname: "silly08"
});

var indexedDB = constructProxy("indexedDB", {
    open: (name) => {
        console.log("opening indexedDB: " + name);
        return DBOpenRequest;
    }
});

var ImageData = constructProxy("ImageData", {
    data: new Uint8Array(128 * 128 * 4)
});

var context = constructProxy("context", {
    createImageData: (w, h) => {
        return ImageData;
    },
    putImageData: (imageData, dx, dy, dirtyX, dirtyY, dirtyWidth, dirtyHeight) => {
        console.log("imageData", dx, dy, dirtyX, dirtyY, dirtyWidth, dirtyHeight);
        let a = new Uint8Array(imageData.data.buffer, imageData.data.byteOffset, imageData.data.length);
        send_data(a);
    }
});

var canvas = constructProxy("canvas", {
    addEventListener: () => { },
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
    addEventListener: () => { },
    getElementById: (id) => {
        console.log("WA: " + id);
        return null;
    },
    activeElement: activeElement,
    currentScript: null,
});
var window = constructProxy("window", {
    addEventListener: () => { console.log("addEventListener") },
    location: Location,
});

var tempDouble, tempI64, el, codo_key_buffer, temp, codo_wheel_dx, codo_wheel_dy, button_i = undefined;

function addEventListener() { }
function setTimeout(callback, time) {
    timeout(time).then(callback);
}