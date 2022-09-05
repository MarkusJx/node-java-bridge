import {
    JavaClassInstance,
    isInstanceOf,
    importClass,
    importClassAsync,
    ensureJvm,
} from '../.';
import { expect } from 'chai';

declare class JString extends JavaClassInstance {
    static valueOf(values: string[]): Promise<JString>;

    static valueOfSync(values: string[]): JString;

    toString(): Promise<string>;

    toStringSync(): string;

    equals(other: JString): Promise<boolean>;

    equalsSync(other: JString): boolean;

    toCharArraySync(): string[];

    toCharArray(): Promise<string[]>;

    getBytesSync(): Buffer;

    splitSync(regex: string): string[];
}

describe('StringTest', () => {
    it('Ensure jvm', () => {
        ensureJvm();
    });

    let JavaString: typeof JString;
    it('Import java.lang.String', () => {
        JavaString = importClass('java.lang.String') as typeof JString;
    });

    it('Cached import', () => {
        importClass('java.lang.String');
    });

    it('Async class resolve', async () => {
        const JavaStringAsync = (await importClassAsync(
            'java.lang.String'
        )) as typeof JString;
        const instance = (await JavaStringAsync.newInstanceAsync(
            'some text'
        )) as JString;

        expect(await instance.toString()).to.equal('some text');
    });

    let s1: JString;
    it('Create string instance', () => {
        s1 = new JavaString('some text');
        expect(s1.toStringSync()).to.equal('some text');
    });

    it('String async match', async function () {
        expect(await s1.toString()).to.equal('some text');
    });

    it('String equals', () => {
        expect(new JavaString('some text').equalsSync(s1)).to.be.true;
        expect(new JavaString('anything').equalsSync(s1)).to.be.false;
    });

    it('String async equals', async () => {
        let s2: JString = new JavaString('some text');
        expect(await s2.equals(s1)).to.be.true;
        expect(await s2.equals(new JavaString('abc'))).to.be.false;
    });

    it('String async create', async function () {
        let s2 = (await JavaString.newInstanceAsync('some text')) as JString;
        expect(await s2.equals(s1)).to.be.true;
    });

    it('String from char array', () => {
        let s2 = JavaString.valueOfSync([
            's',
            'o',
            'm',
            'e',
            ' ',
            't',
            'e',
            'x',
            't',
        ]);
        expect(s2).to.equal(s1.toStringSync());
    });

    it('String to char array', () => {
        let arr = s1.toCharArraySync();
        expect(JSON.stringify(arr)).to.equal(
            JSON.stringify(s1.toStringSync().split(''))
        );
    });

    it('String to char array async', async () => {
        let arr = await s1.toCharArray();
        expect(JSON.stringify(arr)).to.equal(
            JSON.stringify(s1.toStringSync().split(''))
        );
    });

    it('String to byte array', async () => {
        let arr = s1.getBytesSync();
        let bytes = Buffer.from('some text');

        expect(JSON.stringify(arr)).to.equal(JSON.stringify(bytes));
    });

    it('String split', () => {
        const split = s1.splitSync(' ');
        expect(JSON.stringify(split)).to.equal(
            JSON.stringify(['some', 'text'])
        );
    });

    it('InstanceOf', () => {
        const Object = importClass('java.lang.Object');
        expect(s1.instanceOf('java.lang.String')).to.be.true;
        expect(s1.instanceOf('java.lang.Object')).to.be.true;
        expect(s1.instanceOf('java.util.List')).to.be.false;
        expect(s1.instanceOf(JavaString)).to.be.true;
        expect(s1.instanceOf(Object)).to.be.true;

        expect(isInstanceOf(s1, 'java.lang.String')).to.be.true;
        expect(isInstanceOf(s1, 'java.lang.Object')).to.be.true;
        expect(isInstanceOf(s1, 'java.util.List')).to.be.false;
        expect(isInstanceOf(s1, JavaString)).to.be.true;
        expect(isInstanceOf(s1, Object)).to.be.true;
    });
});
