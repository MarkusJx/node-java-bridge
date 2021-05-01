const java = require("./index");
const path = require('path');

process.chdir(__dirname);
console.log(process.cwd());

setTimeout(() => {
    //java.classpath.append(path.join(__dirname, "*"));
    //java.classpath.append("*");
    java.classpath.append("C:/Users/marku/CLionProjects/node-java-bridge/dbLib-1.0-SNAPSHOT.jar");
    //java.createJVM();

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

    console.log(java.javaInstance);

    const SQLite = java.import("io.github.markusjx.database.persistence.SQLiteProvider");
    console.log(SQLite);
}, 100);
