// script that fetches the pico-8 edu JS runtime

(async () => {
    const fs = require('node:fs');

    let baseUrl = "https://www.pico-8-edu.com";
    let outputFile = "crates/engine/src/edu.js";

    let res = await fetch(baseUrl);
    let text = await res.text();

    let pattern = /p8_run_cart\(['"]([a-z \/_.0-9]*)['"].*\)/gm;
    let urlPath;
    try {
        urlPath = [...text.matchAll(pattern)][0][1];
    }
    catch {
        console.error("[ERROR]: Failed to find script source.");
        return;
    }

    let url = baseUrl + urlPath;
    console.log(`Found script url: ${url}`);

    res = await fetch(url);
    let data = await res.bytes();

    fs.writeFileSync(outputFile, data);

    console.log(`Wrote ${outputFile}!`);
})();