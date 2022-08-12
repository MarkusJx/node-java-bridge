import path from 'path';
import { nativeAddon } from './definitions';
import fs from 'fs';

const basePath = path.join(__dirname, '..');

let native: nativeAddon;
if (process.argv.indexOf('--debug') >= 0) {
    console.warn('Running in debug mode');
    native = require(path.join(basePath, 'build', 'Debug', 'node_java_bridge'));
} else {
    native = require(path.join(basePath, 'build', 'Release', 'node_java_bridge'));
}

export const JVM_PATH: string = JSON.parse(
    fs.readFileSync(path.join(basePath, 'jvmLibPath.json'), { encoding: 'utf-8' })
);

let native_path: string;
//process.env.PATH += path.join(basePath, 'build', 'Debug');
if (fs.existsSync(path.join(basePath, 'build', 'Debug', 'node_java_bridge.node'))) {
    native_path = path.join(basePath, 'build', 'Debug', 'node_java_bridge.node');
} else if (fs.existsSync(path.join(basePath, 'build', 'Release', 'node_java_bridge.node'))) {
    native_path = path.join(basePath, 'build', 'Release', 'node_java_bridge.node');
} else {
    throw new Error('Could not find the native binary');
}

native.setNativeLibraryPath(native_path, basePath);

export default native;
