const java = require("./index");

console.log("abc");

setTimeout(() => {
    java.classpath.append("some path");
    java.createJVM();

    const cls = java.load("java.lang.String");
    console.log(Object.getOwnPropertyNames(cls));
    console.log(cls.CASE_INSENSITIVE_ORDER);
    cls.abc = "hello";

    console.log(java.javaInstance);
}, 1000);
