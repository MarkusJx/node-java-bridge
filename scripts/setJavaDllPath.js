const path = require('path');
const fs = require('fs');
const {findJavaLibrary} = require("./findJavaLibrary");

const outFile = path.join(__dirname, '..', 'jvmLibPath.json');

async function run() {
    const path = await findJavaLibrary(true);
    fs.writeFileSync(outFile, JSON.stringify(path), {flag: 'w', encoding: 'utf-8'});
}

run().then().catch(e => {
    console.error(e);
    process.exit(1);
});