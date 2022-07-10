import findJavaHome from 'find-java-home';
import path from 'path';
import fs from 'fs';

/**
 * Find the java home path
 *
 * @param {boolean} allowJre whether to allow runtime environments
 * @returns {Promise<string>} the java home path
 */
export function findHome(allowJre = true): Promise<string> {
    return new Promise((resolve, reject) => {
        findJavaHome({ allowJre }, (err, res) => {
            if (err) {
                reject(err);
            } else {
                resolve(res);
            }
        }).then();
    });
}

/**
 * Find the java library
 *
 * @param {boolean} allowJre whether to allow runtime environments
 * @returns {Promise<string>} the dll path
 */
export async function findJavaLibrary(allowJre = true): Promise<string> {
    const home = await findHome(allowJre);
    let libPath = 'lib';
    if (process.platform === 'win32') {
        libPath = 'bin';
    }

    let libraryName;
    if (process.platform === 'win32') {
        libraryName = 'jvm.dll';
    } else if (process.platform === 'darwin') {
        libraryName = 'libjvm.dylib';
    } else {
        libraryName = 'libjvm.so';
    }

    const client = path.join(home, libPath, 'client', libraryName);
    const server = path.join(home, libPath, 'server', libraryName);

    if (fs.existsSync(client)) {
        return client;
    } else if (fs.existsSync(server)) {
        return server;
    } else {
        throw new Error(
            `It looks like java is not installed: Neither '${client}' nor '${server}' exists, cannot continue`
        );
    }
}
