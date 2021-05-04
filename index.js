let native;
if (process.argv.indexOf("--debug") >= 0) {
    console.warn("Running in debug mode");
    native = require('./build/Debug/node_java_bridge');
} else {
    native = require('./build/Release/node_java_bridge');
}

const JVM_PATH = "C:\\Program Files\\AdoptOpenJDK\\jdk-11.0.10.9-hotspot\\bin\\client\\jvm.dll";
//const JVM_PATH = "C:\\Users\\marku\\Desktop\\jdk\\build\\windows-x86_64-server-fastdebug\\jdk\\bin\\server\\jvm.dll";

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