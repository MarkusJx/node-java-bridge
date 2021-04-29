const java = require("./index");

setTimeout(() => {
    java.classpath.append("some path");
    java.createJVM();

    const cls = java.import("java.lang.String");
    //console.log(cls);
    //console.log(cls['CASE_INSENSITIVE_ORDER']);

    const instance = new cls("abc");
    //console.log(JSON.stringify(instance));
    //console.log(Object.getOwnPropertyNames(instance));
    //console.log(instance);
    console.log(instance.charAt(0));
    console.log(instance.charAt(0));
    console.log(instance.indexOf('a'));
    console.log(cls.valueOf(25));

    //cls.abc = "hello";
    //cls.CASE_INSENSITIVE_ORDER = "abc";

    //console.log(java.javaInstance);
}, 100);
