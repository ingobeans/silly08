var documentItems = {
    addEventListener: () => { },
    getElementById: (id) => {
        console.log("WA: " + id);
        return null;
    },
    get activeElement() {
        console.log("wa");
        return {}
    },
};

var ModuleItems = {
    canvas: {},
}


var Module = new Proxy({}, {
    get(target, property) {
        console.log("Module " + property);
        return property in target ? target[property] : undefined;
    }
});
for (let [k, v] in Object.entries(ModuleItems)) {
    Module[k] = v;
}
var document = new Proxy({}, {
    get(target, property) {
        console.log("document " + property);
        return property in target ? target[property] : undefined;
    }
});
for (let [k, v] in Object.entries(documentItems)) {
    document[k] = v;
}

var window = {
    addEventListener: () => { console.log("addEventListener") },
};
var el = undefined;
var codo_key_buffer = undefined;
function addEventListener() { }
