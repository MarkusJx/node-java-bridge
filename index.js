let native;
if (process.argv.indexOf("--debug") >= 0) {
    console.warn("Running in debug mode");
    native = require('./build/Debug/node_java_bridge');
} else {
    native = require('./build/Release/node_java_bridge');
}

const JVM_PATH = "C:\\Program Files\\AdoptOpenJDK\\jdk-11.0.10.9-hotspot\\bin\\client\\jvm.dll";

let java = new native.java(JVM_PATH, "1.6");

module.exports = {
    classpath: {
        append: function(path) {
            java.appendToClasspath(path);
        }
    },
    createJVM: function(version, jvmPath = null) {
        java = new native.java(JVM_PATH, version);
    },
    importClass: function (classname) {
        return java.getClass(classname).getClassConstructor();
    },
    getJavaInstance: function () {
        return java;
    }
};