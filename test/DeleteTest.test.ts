import { importClass, deleteObject, newProxy } from '../.';
import { expect } from 'chai';
import {
    shouldIncreaseTimeout,
    getUsedMemory,
    generateRandomString,
} from './testUtil';
import {
    RuntimeClass,
    SystemClass,
    JString as StringClass,
    FunctionInterface,
} from './classes';
import isCi from 'is-ci';

const timeout = shouldIncreaseTimeout ? 120e3 : 20e3;

describe('DeleteTest', () => {
    it('Delete string instance', () => {
        const JString = importClass<typeof StringClass>('java.lang.String');
        const System = importClass<typeof SystemClass>('java.lang.System');
        const Runtime = importClass<typeof RuntimeClass>('java.lang.Runtime');

        System.gcSync();
        const before = getUsedMemory(Runtime);
        const string = new JString(Buffer.alloc(1024 * 1024 * 10));

        const after = getUsedMemory(Runtime);
        expect(after).to.be.greaterThan(before);

        deleteObject(string);
        System.gcSync();
        const end = getUsedMemory(Runtime);
        expect(end).to.be.lessThan(after - 10_000_000);
    })
        .timeout(timeout)
        .retries(3);

    it('Delete deleted instance', () => {
        const JString = importClass<typeof StringClass>('java.lang.String');
        const string = new JString('Hello World');
        deleteObject(string);
        expect(() => deleteObject(string)).to.throw();
    }).timeout(timeout);

    it('Access deleted instance', () => {
        const JString = importClass<typeof StringClass>('java.lang.String');
        const string = new JString('Hello World');
        deleteObject(string);
        expect(() => string.toString()).to.throw();
    }).timeout(timeout);

    it('Check proxy memory leak', async function () {
        if (isCi && (process.arch === 'arm64' || process.arch === 'arm')) {
            this.skip();
        }

        const Runtime = importClass<typeof RuntimeClass>('java.lang.Runtime');
        const System = importClass<typeof SystemClass>('java.lang.System');
        const JString = importClass<typeof StringClass>('java.lang.String');

        const string = new JString();

        System.gcSync();
        const before = getUsedMemory(Runtime);
        const proxy = newProxy<FunctionInterface<string>>(
            'java.util.function.Function',
            {
                apply: () => {
                    return generateRandomString(1024 * 1024 * 10);
                },
            }
        );

        System.gcSync();
        await string.transform(proxy);
        const after = getUsedMemory(Runtime);
        expect(after).to.be.greaterThan(before);

        proxy.reset();
        deleteObject(string);
        System.gcSync();
        const end = getUsedMemory(Runtime);
        expect(end).to.be.lessThanOrEqual(after - 10_000_000);
    }).timeout(timeout);
});
