import {
    isInstanceOf,
    importClass,
    importClassAsync,
    ensureJvm,
    config,
    clearClassProxies,
} from '../.';
import { expect, use } from 'chai';
import { inspect } from 'util';
import { JString } from './classes';
import chaiAsPromised from 'chai-as-promised';

use(chaiAsPromised);

describe('StringTest', () => {
    it('Ensure jvm', () => {
        ensureJvm();
    });

    let JavaString: typeof JString;
    it('Import java.lang.String', () => {
        JavaString = importClass<typeof JString>('java.lang.String');
    });

    it('Cached import', () => {
        importClass('java.lang.String');
    });

    it('Async class resolve', async () => {
        const JavaStringAsync =
            await importClassAsync<typeof JString>('java.lang.String');
        const instance = await JavaStringAsync.newInstanceAsync('some text');

        expect(await instance.toStringAsync()).to.equal('some text');
    });

    let s1: JString;
    it('Create string instance', () => {
        s1 = new JavaString('some text');
        expect(s1.toStringSync()).to.equal('some text');
    });

    it('String async match', async function () {
        expect(await s1.toStringAsync()).to.equal('some text');
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
            JSON.stringify(s1.toString().split(''))
        );
    });

    it('String to char array async', async () => {
        let arr = await s1.toCharArray();
        expect(JSON.stringify(arr)).to.equal(
            JSON.stringify(s1.toString().split(''))
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

    it('toString', async () => {
        const str = new JavaString('test');

        expect(str.toString()).to.equal('test');
        expect(await str.toStringAsync()).to.equal('test');
        expect(str + '').to.equal('test');
    });

    it('inspect.custom', () => {
        clearClassProxies();
        config.customInspect = true;
        const LoggableString = importClass<typeof JString>('java.lang.String');
        const str = new LoggableString('test');
        config.customInspect = false;

        expect(str[inspect.custom]!()).to.equal('test');
    });

    it('async stack trace', async () => {
        await expect(JavaString.newInstanceAsync(null))
            .to.eventually.be.rejected.property('stack')
            .to.match(
                /at Context\.<anonymous> \(.+test[/\\]StringTest\.test\.ts:\d+:\d+\)/gm
            );
    });
});
