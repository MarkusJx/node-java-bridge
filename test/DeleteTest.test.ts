import { importClass, deleteObject, JavaClass } from '../.';
import { expect } from 'chai';
import { shouldIncreaseTimeout } from './testUtil';

const timeout = shouldIncreaseTimeout ? 60e3 : 20e3;

declare class RuntimeClass extends JavaClass {
    public static getRuntimeSync(): RuntimeClass;

    public totalMemorySync(): bigint;

    public freeMemorySync(): bigint;
}

const getUsedMemory = (Runtime: typeof RuntimeClass): number => {
    return Number(
        Runtime.getRuntimeSync().totalMemorySync() -
            Runtime.getRuntimeSync().freeMemorySync()
    );
};

describe('DeleteTest', () => {
    it('Delete string instance', function () {
        if (shouldIncreaseTimeout) this.timeout(timeout);

        const String = importClass('java.lang.String');
        const System = importClass('java.lang.System');
        const Runtime = importClass<typeof RuntimeClass>('java.lang.Runtime');

        System.gcSync();
        const before = getUsedMemory(Runtime);
        const string = new String(Buffer.alloc(1024 * 1024 * 10));

        const after = getUsedMemory(Runtime);
        expect(after).to.be.greaterThan(before);

        deleteObject(string);
        System.gcSync();
        const end = getUsedMemory(Runtime);
        expect(end).to.be.lessThan(after);
    });

    it('Delete deleted instance', function () {
        if (shouldIncreaseTimeout) this.timeout(timeout);

        const String = importClass('java.lang.String');
        const string = new String('Hello World');
        deleteObject(string);
        expect(() => deleteObject(string)).to.throw();
    });

    it('Access deleted instance', function () {
        if (shouldIncreaseTimeout) this.timeout(timeout);

        const String = importClass('java.lang.String');
        const string = new String('Hello World');
        deleteObject(string);
        expect(() => string.toString()).to.throw();
    });
});
