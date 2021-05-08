import java, {java_instance_proxy} from "../index";
import assert = require("assert");
import LogLevel = java.logging.LogLevel;

declare class JString extends java_instance_proxy {
    static valueOf(values: string[]): Promise<JString>;

    static valueOfSync(values: string[]): JString;

    toString(): Promise<string>;

    toStringSync(): string;

    equals(other: JString): Promise<boolean>;

    equalsSync(other: JString): boolean;

    toCharArraySync(): string[];

    toCharArray(): Promise<string[]>;

    getBytesSync(): number[];

    splitSync(regex: string): string[];
}

java.logging.setLogLevel(LogLevel.WARNING);

describe('StringTest', () => {
    it('Ensure jvm', () => {
        java.ensureJVM();
    });

    let JavaString: typeof JString;
    it('Import java.lang.String', () => {
        JavaString = java.importClass('java.lang.String') as typeof JString;
    });

    it('Cached import', () => {
        java.importClass('java.lang.String');
    })

    it('Async class resolve', async () => {
        const JavaStringAsync = await java.importClassAsync('java.lang.String') as typeof JString;
        const instance = await JavaStringAsync.newInstance('some text') as JString;

        assert.strictEqual(await instance.toString(), "some text");
    });

    let s1: JString;
    it('Create string instance', () => {
        s1 = new JavaString("some text");
        assert.strictEqual(s1.toStringSync(), "some text");
    });

    it('String async match', async function () {
        assert.strictEqual(await s1.toString(), "some text");
    });

    it('String equals', () => {
        assert.strictEqual(new JavaString("some text").equalsSync(s1), true);
        assert.strictEqual(new JavaString("anything").equalsSync(s1), false);
    });

    it('String async equals', async () => {
        let s2: JString = new JavaString("some text");
        assert.strictEqual(await s2.equals(s1), true);
        assert.strictEqual(await s2.equals(new JavaString("abc")), false);
    });

    it('String async create', async function () {
        let s2 = await JavaString.newInstance("some text") as JString;
        assert.strictEqual(await s2.equals(s1), true);
    });

    it('String from char array', () => {
        let s2 = JavaString.valueOfSync(['s', 'o', 'm', 'e', ' ', 't', 'e', 'x', 't']);
        assert.strictEqual(s2, s1.toStringSync());
    });

    it('String to char array', () => {
        let arr = s1.toCharArraySync();
        assert.strictEqual(JSON.stringify(arr), JSON.stringify(s1.toStringSync().split("")));
    });

    it('String to char array async', async () => {
        let arr = await s1.toCharArray();
        assert.strictEqual(JSON.stringify(arr), JSON.stringify(s1.toStringSync().split("")));
    })

    it('String to byte array', async () => {
        let arr = s1.getBytesSync();
        let bytes = [...Buffer.from("some text")];

        assert.strictEqual(JSON.stringify(arr), JSON.stringify(bytes));
    });

    it('String split', () => {
        const split = s1.splitSync(" ");
        assert.strictEqual(JSON.stringify(split), JSON.stringify(["some", "text"]));
    });
});