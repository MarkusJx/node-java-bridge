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
    getJavaInstance: function () {
        return java;
    },
    ensureJVM: ensureJVM,
    reset: function() {
        java = null;
    }
};