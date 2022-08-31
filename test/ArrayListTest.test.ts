import java, {
    JavaClassInstance,
    JavaConstructor,
    JavaType,
    isInstanceOf,
} from '../.';
import assert = require('assert');
import { it } from 'mocha';
import { expect } from 'chai';

declare class ListClass<T extends JavaType> extends JavaClassInstance {
    containsSync(element: T): boolean;
    sizeSync(): number;
    getSync(index: number): T;
    lastIndexOfSync(element: T): number;
    addSync(value: T): void;
    removeSync(index: number): T;
    toArraySync(): T[];
    isEmptySync(): boolean;

    add(value: T): Promise<void>;
    lastIndexOf(element: T): Promise<number>;
    contains(element: T): Promise<boolean>;
    isEmpty(): Promise<boolean>;
    size(): Promise<number>;
    get(index: number): Promise<T>;
    remove(index: number): Promise<T>;
}

declare class ArrayListClass<T extends JavaType> extends ListClass<T> {}

describe('ArrayListTest', () => {
    let list: ArrayListClass<JavaType> | null = null;
    let ArrayList: JavaConstructor<typeof ArrayListClass<JavaType>> | null =
        null;

    it('Ensure jvm', () => {
        java.ensureJvm();
    }).timeout(10000);

    it('Import java.util.ArrayList', () => {
        ArrayList = java.importClass<typeof ArrayListClass>(
            'java.util.ArrayList'
        )<JavaType>;
    });

    it('Create a new ArrayList', () => {
        list = new ArrayList!();
        assert.notStrictEqual(list, null);
        assert.strictEqual(list!.isEmptySync(), true);
    });

    it('Get the list size', () => {
        assert.strictEqual(list!.sizeSync(), 0);
    });

    const elements: any[] = [];

    it('Insert int', () => {
        list!.addSync(123);
        elements.push(123);

        assert.strictEqual(list!.containsSync(123), true);
        assert.strictEqual(list!.sizeSync(), 1);
        assert.strictEqual(list!.getSync(0), 123);
    });

    it('Insert string', () => {
        const val = 'Some string';
        list!.addSync(val);
        elements.push(val);

        assert.strictEqual(list!.lastIndexOfSync(val), 1);
        assert.strictEqual(list!.sizeSync(), 2);
        assert.strictEqual(list!.getSync(1), val);
    });

    it('Insert long', () => {
        let long = BigInt(2313213213);
        list!.addSync(long);
        elements.push(long);

        assert.strictEqual(list!.sizeSync(), 3);
        assert.strictEqual(list!.getSync(2), long);
    });

    it('Insert double async', async () => {
        const double = 12.21341;
        await list!.add(double);
        elements.push(double);

        assert.strictEqual(await list!.lastIndexOf(double), 3);
        assert.strictEqual(await list!.contains(double), true);
        assert.strictEqual(await list!.isEmpty(), false);
        assert.strictEqual(await list!.size(), 4);
        assert.strictEqual(await list!.get(3), double);
    });

    it('Insert Float', () => {
        const Float = java.importClass('java.lang.Float');
        const val = new Float(1232.248);

        list!.addSync(val);
        elements.push(val.doubleValueSync());
    });

    it('toArray', () => {
        // Create a conversion helper as json
        // doesn't know how to serialize a BigInt
        const converter = (key: any, value: any) => {
            if (typeof value === 'bigint') {
                return value.toString();
            } else {
                return value;
            }
        };

        assert.strictEqual(
            JSON.stringify(list!.toArraySync(), converter),
            JSON.stringify(elements, converter)
        );
    });

    it('List remove', () => {
        assert.strictEqual(list!.removeSync(0), 123);
        assert.strictEqual(list!.sizeSync(), 4);
    });

    it('List remove async', async () => {
        assert.strictEqual(await list!.remove(0), 'Some string');
        assert.strictEqual(await list!.size(), 3);
    });

    let list_cpy: ArrayListClass<JavaType> | null = null;

    it('List copy', () => {
        list_cpy = new ArrayList!(list!);

        assert.strictEqual(list!.sizeSync(), list_cpy!.sizeSync());
    });

    it('List clear', () => {
        list!.clearSync();
        assert.strictEqual(list!.isEmptySync(), true);
        assert.strictEqual(list!.sizeSync(), 0);
    });

    it('List clear async', async () => {
        await list_cpy!.clear();
        assert.strictEqual(await list_cpy!.isEmpty(), true);
        assert.strictEqual(await list_cpy!.size(), 0);
    });

    it('InstanceOf', () => {
        const List = java.importClass<typeof ListClass>('java.util.List');
        const Object = java.importClass('java.lang.Object');
        const JavaString = java.importClass('java.lang.String');

        expect(list!.instanceOf(ArrayList!)).to.be.true;
        expect(list!.instanceOf(List)).to.be.true;
        expect(list!.instanceOf(Object)).to.be.true;
        expect(list!.instanceOf(JavaString)).to.be.false;
        expect(list!.instanceOf('java.util.List')).to.be.true;
        expect(list!.instanceOf('java.util.ArrayList')).to.be.true;
        expect(list!.instanceOf('java.lang.Object')).to.be.true;
        expect(list!.instanceOf('java.lang.String')).to.be.false;

        expect(isInstanceOf(list!, ArrayList!)).to.be.true;
        expect(isInstanceOf(list!, List)).to.be.true;
        expect(isInstanceOf(list!, Object)).to.be.true;
        expect(isInstanceOf(list!, JavaString)).to.be.false;
        expect(isInstanceOf(list!, 'java.util.List')).to.be.true;
        expect(isInstanceOf(list!, 'java.util.ArrayList')).to.be.true;
        expect(isInstanceOf(list!, 'java.lang.Object')).to.be.true;
        expect(isInstanceOf(list!, 'java.lang.String')).to.be.false;
    });
});
