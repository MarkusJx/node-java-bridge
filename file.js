/*const {Java, getClassFields, getStaticField} = require("./index");

let java = new Java();
const String = java.importClass("java.lang.String");
console.log(java.importClassAsync("java.lang.String"));
console.log("Class:", String);
console.log("Instance:", new String("test").charAt(1).then(console.log));
console.log("Instance:", String.newInstance("test").then(s => s.charAt(1).then(console.log)));
console.log(getClassFields(String["class.proxy"], true));
console.log(getStaticField(String, "CASE_INSENSITIVE_ORDER"));*/

const { ensureJvm, importClass, JavaVersion, stdout, newProxy } = require('.');

ensureJvm(undefined, JavaVersion.VER_10);

let JString = importClass('java.lang.String');
let a = JString.valueOfSync(['s', 'o', 'm', 'e', ' ', 't', 'e', 'x', 't']);
console.log(a, typeof a);
//console.log(a.toStringSync());
console.log(JString);
console.log(new JString('test').toStringSync());
console.log(JString.valueOfSync('test'));

let redirect = stdout.enableRedirect((_, data) => console.log('data:', data));
const System = importClass('java.lang.System');
System.out.printlnSync('Hello World');
redirect.on('stdout', (_, data) => console.log('data1:', data));
System.out.printlnSync('abc');
redirect.reset();

const proxy = newProxy('java.util.function.Function', {
    apply: (arg) => {
        console.log('arg:', arg);
        return arg.toUpperCase();
    },
});

new JString('test').transform(proxy).then((a) => console.log('res:', a));

/*function err() {
    throw new Error('error');
}

const Thread = importClass('java.lang.Thread');
let proxy = newProxy('java.lang.Runnable', {
    run: () => {
        console.log('ok');
        err();
        //done();
    },
});

console.log(proxy);

let thread = new Thread(proxy);
thread.startSync();
thread.join().then(() => {
    console.log('reset');
    proxy.reset();
    proxy = null;
});

/*const ArrayList = importClass('java.util.ArrayList');
const list = new ArrayList();

//list.addSync(123);
//console.log(list.getSync(0));

let long = BigInt(2313213213);
const Float = importClass('java.lang.Float');
const val = new Float(1232.248);
console.log(val)
list.addSync(val);

console.log(list.getSync(0))*/
