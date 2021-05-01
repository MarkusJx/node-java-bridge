const java = require("./index");

setTimeout(() => {
    java.classpath.append("C:/Users/marku/CLionProjects/node-java-bridge/dbLib-1.0-SNAPSHOT.jar");

    const cls = java.importClass("java.lang.String");
    //console.log(cls['CASE_INSENSITIVE_ORDER']);

    const instance = new cls("abc");
    //console.log(JSON.stringify(instance));
    //console.log(Object.getOwnPropertyNames(instance));
    console.log(instance.charAt(0));
    console.log(instance.charAt(0));
    console.log(instance.indexOf('a'));
    console.log(cls.valueOf(25));

    //cls.abc = "hello";
    //cls.CASE_INSENSITIVE_ORDER = "abc";

    console.log(java.getJavaInstance());
    console.log(java.getJavaInstance().loadedJars);

    const SQLite = java.importClass("io.github.markusjx.database.persistence.SQLiteProvider");
    console.log(SQLite);
}, 100);
