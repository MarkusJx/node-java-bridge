const path = require('path');
const fs = require('fs');

let native;
if (process.argv.indexOf("--debug") >= 0) {
    console.warn("Running in debug mode");
    native = require('./build/Debug/node_java_bridge');
} else {
    native = require('bindings')('node_java_bridge.node');
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
    reset: function() {
        java = null;
    },
    logging: {
        setLogLevel: function(level) {
            native.setLoggerMode(level);
        },
        LogLevel: LogLevel
    }
};