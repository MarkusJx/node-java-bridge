const java = require("./index");



setTimeout(() => {
    java.ensureJVM();
    java.classpath.append("C:/Users/marku/CLionProjects/node-java-bridge/dbLib-1.0-SNAPSHOT.jar");
    //return;

    const cls = java.importClass("java.lang.String");
    //return;
    //console.log(cls['CASE_INSENSITIVE_ORDER']);
    //return;
    //console.log(Object.getOwnPropertyNames(cls));
    cls.newInstance(123).then(e => console.log(e), e => console.error(e));
    //return;

    const instance = new cls("abc");
    //return;
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
    const action = java.importClass("org.hibernate.tool.schema.Action");
    const provider = new SQLite("", action.NONE, false, ["abc"]);

    console.log(Object.getOwnPropertyNames(provider));

    const CustomPersistence = java.importClass("io.github.markusjx.database.persistence.CustomPersistence");
    const DatabaseManager = java.importClass("io.github.markusjx.database.DatabaseManager");
    const factory = CustomPersistence.createEntityManagerFactory("documents", provider);
    const em = factory.createEntityManager();
    const manager = new DatabaseManager(em);
    manager.close();

    console.log(Object.getOwnPropertyNames(manager));
}, 100);
