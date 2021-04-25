const java = require("./index");

console.log("abc");

setTimeout(() => {
    java.classpath.append("some path");
    java.createJVM();

    const cls = java.load("java.lang.String");
    console.log(cls);
    console.log(cls['CASE_INSENSITIVE_ORDER']);

    const instance = new cls();
    console.log(instance);
    //cls.abc = "hello";
    //cls.CASE_INSENSITIVE_ORDER = "abc";

    console.log(java.javaInstance);
}, 1000);
