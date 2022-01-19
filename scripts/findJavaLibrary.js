const findJavaHome = require("find-java-home");
const path = require("path");
const fs = require("fs");

/**
 * Find the java home path
 *
 * @param {boolean} allowJre whether to allow runtime environments
 * @returns {Promise<string>} the java home path
 */
function findHome(allowJre = true) {
    return new Promise((resolve, reject) => {
        findJavaHome({allowJre}, (err, res) => {
            if (err) {
                reject(err);
            } else {
                resolve(res);
            }
        });
    });
}

/**
 * Find the java library
 *
 * @param {boolean} allowJre whether to allow runtime environments
 * @returns {Promise<string>} the dll path
 */
async function findJavaLibrary(allowJre = true) {
    const home = await findHome(allowJre);
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
        return client;
    } else if (fs.existsSync(server)) {
        return server;
    } else {
        throw new Error(`It looks like java is not installed: Neither '${client}' nor '${server}' exists, cannot continue`);
    }
}

module.exports = {
    findHome,
    findJavaLibrary
};