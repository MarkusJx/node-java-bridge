const java = require("./index");

java.logging.setLogLevel(3);

setTimeout(() => {
    java.ensureJVM();
    //java.classpath.append("C:/Users/marku/CLionProjects/node-java-bridge/dbLib-1.0-SNAPSHOT.jar");
    //return;

    /*nlet proxy = java.newProxy('java.lang.Runnable', {
        run: () => {
            console.log("Run called");
        }
    });

    let Thread = java.importClass('java.lang.Thread');
    let thread = new Thread(proxy);
    thread.startSync();*/

    //return;

    const cls = java.importClass("java.lang.String");

    java.stdoutRedirect.enableRedirect(msg => {
        console.log(msg);
    });

    const system = java.importClass("java.lang.System");
    system.out.printlnSync("test");
    java.stdoutRedirect.reset();
    //return;
    //console.log(cls['CASE_INSENSITIVE_ORDER']);
    //return;
    //console.log(Object.getOwnPropertyNames(cls));
    //cls.newInstance(123).then(e => console.log(e), e => console.error(e));
    //return;

    /*const instance = new cls("abc");
    //return;
    //console.log(JSON.stringify(instance));
    //console.log(Object.getOwnPropertyNames(instance));
    console.log(instance.charAt(0).then(console.log));
    console.log(instance.charAt(0).then(console.log));
    console.log(instance.indexOf('a').then(console.log));
    console.log(cls.valueOf(25).then(e => console.log("val", e)).catch(console.error));
    console.log(instance.toString().then(console.log));
    console.log(instance.toCharArraySync());
    //return;

    //cls.abc = "hello";
    //cls.CASE_INSENSITIVE_ORDER = "abc";

    //console.log(java.getJavaInstance());
    //console.log(java.getJavaInstance().loadedJars);

    const SQLite = java.importClass("io.github.markusjx.database.persistence.SQLiteProvider");
    const action = java.importClass("org.hibernate.tool.schema.Action");
    const provider = new SQLite("", action.NONE, false, ["abc"]);

    //console.log(Object.getOwnPropertyNames(provider));

    const CustomPersistence = java.importClass("io.github.markusjx.database.persistence.CustomPersistence");
    const DatabaseManager = java.importClass("io.github.markusjx.database.DatabaseManager");
    console.log(Object.getOwnPropertyNames(CustomPersistence));
    const factory = CustomPersistence.createEntityManagerFactorySync("documents", provider);
    const em = factory.createEntityManagerSync();
    const manager = new DatabaseManager(em);
    manager.closeSync();

    //console.log(Object.getOwnPropertyNames(manager));*/
}, 1000);
