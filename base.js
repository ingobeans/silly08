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

var context = constructProxy("context", {});

var canvas = constructProxy("canvas", {
    addEventListener: () => { },
    getContext: () => { context },
});

var Module = constructProxy("Module", {
    canvas: canvas,
    onAbort: (what) => { console.log("ABORT: " + what) }
});
var document = constructProxy("document", {
    addEventListener: () => { },
    getElementById: (id) => {
        console.log("WA: " + id);
        return null;
    },
    get activeElement() {
        console.log("wa");
        return {}
    },
    currentScript: null,
});
var window = constructProxy("window", {
    addEventListener: () => { console.log("addEventListener") },
    location: Location,
});

var tempDouble = undefined;
var tempI64 = undefined;
var el = undefined;
var codo_key_buffer = undefined;

function addEventListener() { }
function setTimeout(callback, time) {
    console.log("setTimeout " + time);
}