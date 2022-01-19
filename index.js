const path = require('path');
const fs = require('fs');
const {findJavaLibrary} = require("./scripts/findJavaLibrary");

let native;
if (process.argv.indexOf("--debug") >= 0) {
    console.warn("Running in debug mode");
    native = require('./build/Debug/node_java_bridge');
} else {
    native = require('./build/Release/node_java_bridge.node');
}

const JVM_PATH = JSON.parse(fs.readFileSync(path.join(__dirname, 'jvmLibPath.json'), {encoding: 'utf-8'}));

const java_version = {
    VER_1_1: "1.1",
    VER_1_2: "1.2",
    VER_1_4: "1.4",
    VER_1_6: "1.6",
    VER_1_8: "1.8",
    VER_9: "9",
    VER_10: "10"
};

Object.freeze(java_version);

const LogLevel = {
    DEBUG: 0,
    WARNING: 1,
    ERROR: 2,
    NONE: 3
};

Object.freeze(LogLevel);

function ensureJVM(jvmPath = null, version = null) {
    if (java == null) {
        if (jvmPath == null) {
            jvmPath = JVM_PATH;
        }

        java = new native.java(jvmPath, version);
    }
}

let native_path;
process.env.PATH += path.join(__dirname, 'build', 'Debug');
if (fs.existsSync(path.join(__dirname, 'build', 'Debug', 'node_java_bridge.node'))) {
    native_path = path.join(__dirname, 'build', 'Debug', 'node_java_bridge.node');
} else if (fs.existsSync(path.join(__dirname, 'build', 'Release', 'node_java_bridge.node'))) {
    native_path = path.join(__dirname, 'build', 'Release', 'node_java_bridge.node');
} else {
    throw new Error("Could not find the native binary");
}

const functionCallerPath = path.join(__dirname, 'io', 'github', 'markusjx', 'bridge', 'JavaFunctionCaller.class');
const functionCaller = fs.readFileSync(functionCallerPath);
native.setNativeLibraryPath(native_path, functionCaller);

let java = null;

module.exports = {
    java_version: java_version,
    classpath: {
        append: function (path) {
            ensureJVM();
            java.appendToClasspath(path);
        },
        appendAsync: function(path) {
            ensureJVM();
            return java.appendToClasspathAsync(path);
        }
    },
    createJVM: function (jvmPath = null, version = null) {
        native.java.destroyJVM();
        java = null;
        ensureJVM(jvmPath, version);
    },
    importClass: function (classname) {
        ensureJVM();
        return native.java.getClass(classname).getClassConstructor();
    },
    importClassAsync: async function(classname) {
        ensureJVM();
        const proxy = await native.java.getClassAsync(classname);
        return proxy.getClassConstructor();
    },
    getJavaInstance: function () {
        return java;
    },
    ensureJVM: ensureJVM,
    destroyJVM: function () {
        native.java.destroyJVM();
    },
    findJVM: findJavaLibrary,
    logging: {
        setLogLevel: function(level) {
            native.setLoggerMode(level);
        },
        LogLevel: LogLevel
    },
    newProxy: function (name, obj) {
        ensureJVM();
        return new native.java_function_caller(name, obj);
    }
};