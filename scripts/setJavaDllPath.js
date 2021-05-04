const findJavaHome = require('find-java-home');
const path = require('path');
const fs = require('fs');

function findHome() {
    return new Promise((resolve, reject) => {
        findJavaHome({allowJre: true}, (err, res) => {
            if (err) {
                reject(err);
            } else {
                resolve(res);
            }
        });
    });
}

const outFile = path.join(__dirname, '..', 'jvmLibPath.json');

async function run() {
    const home = await findHome();
    let libPath = 'lib';
    if (process.platform === 'win32') {
        libPath = 'bin';
    }

    let libraryName;
    if (process.platform === 'win32') {
        libraryName = "jvm.dll";
    } else if (process.platform === 'darwin') {
        libraryName = "libjvm.dylib";
    } else {
        libraryName = "libjvm.so";
    }

    const client = path.join(home, libPath, 'client', libraryName);
    const server = path.join(home, libPath, 'server', libraryName);

    if (fs.existsSync(client)) {
        fs.writeFileSync(outFile, JSON.stringify(client), {flag: 'w', encoding: 'utf-8'});
    } else if (fs.existsSync(server)) {
        fs.writeFileSync(outFile, JSON.stringify(server), {flag: 'w', encoding: 'utf-8'});
    } else {
        throw new Error(`Neither '${client}' nor '${server}' exists, cannot continue`);
    }
}

run().then();