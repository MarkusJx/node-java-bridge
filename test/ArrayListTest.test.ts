import java, { JavaClass, JavaType, isInstanceOf } from '../.';
import { it } from 'mocha';
import { expect } from 'chai';
import { shouldIncreaseTimeout } from './testUtil';

const timeout = shouldIncreaseTimeout ? 20e3 : 2e3;

declare class StreamClass<T extends JavaType> extends JavaClass {
    toListSync(): ListClass<T>;
    toList(): Promise<ListClass<T>>;
}

declare class ListClass<T extends JavaType> extends JavaClass {
    containsSync(element: T): boolean;
    sizeSync(): number;
    getSync(index: number): T;
    lastIndexOfSync(element: T): number;
    addSync(value: T): void;
    removeSync(index: number): T;
    toArraySync(): T[];
    isEmptySync(): boolean;
    clearSync(): void;

    add(value: T): Promise<void>;
    lastIndexOf(element: T): Promise<number>;
    contains(element: T): Promise<boolean>;
    isEmpty(): Promise<boolean>;
    size(): Promise<number>;
    get(index: number): Promise<T>;
    remove(index: number): Promise<T>;
    clear(): Promise<void>;

    streamSync(): StreamClass<T>;
    stream(): Promise<StreamClass<T>>;
}

declare class ArrayListClass<T extends JavaType> extends ListClass<T> {
    static newInstanceAsync(): Promise<ArrayListClass<JavaType>>;

    constructor();
    constructor(other: ListClass<T>);
}

describe('ArrayListTest', () => {
    let list: ArrayListClass<JavaType> | null = null;
    let ArrayList: typeof ArrayListClass<JavaType> | null = null;

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
        expect(list).to.not.be.null;
        expect(list!.isEmptySync()).to.be.true;
    });

    it('Get the list size', () => {
        expect(list!.sizeSync()).to.equal(0);
    });

    const elements: any[] = [];

    it('Insert int', () => {
        list!.addSync(123);
        elements.push(123);

        expect(list!.containsSync(123)).to.be.true;
        expect(list!.sizeSync()).to.equal(1);
        expect(list!.getSync(0)).to.equal(123);
    });

    it('Insert string', () => {
        const val = 'Some string';
        list!.addSync(val);
        elements.push(val);

        expect(list!.lastIndexOfSync(val)).to.equal(1);
        expect(list!.sizeSync()).to.equal(2);
        expect(list!.getSync(1)).to.equal(val);
    });

    it('Insert long', () => {
        let long = BigInt(2313213213);
        list!.addSync(long);
        elements.push(long);

        expect(list!.sizeSync()).to.equal(3);
        expect(list!.getSync(2)).to.equal(long);
    });

    it('Insert double async', async () => {
        const double = 12.21341;
        await list!.add(double);
        elements.push(double);

        expect(await list!.lastIndexOf(double)).to.equal(3);
        expect(await list!.contains(double)).to.be.true;
        expect(await list!.isEmpty()).to.be.false;
        expect(await list!.size()).to.equal(4);
        expect(await list!.get(3)).to.equal(double);
    });

    it('Insert Float', () => {
        const Float = java.importClass('java.lang.Float');
        const val = new Float(1232.248);

        list!.addSync(val);
        elements.push(val.doubleValueSync());

        expect(list!.sizeSync()).to.equal(5);
        expect(list!.getSync(4)).to.equal(val.doubleValueSync());
    });

    it('toArray', () => {
        // Create a conversion helper as json
        // doesn't know how to serialize a BigInt
        const converter = (_: any, value: any) => {
            if (typeof value === 'bigint') {
                return value.toString();
            } else {
                return value;
            }
        };

        expect(JSON.stringify(list!.toArraySync(), converter)).to.equal(
            JSON.stringify(elements, converter)
        );
    });

    it('List remove', () => {
        expect(list!.removeSync(0)).to.equal(123);
        expect(list!.sizeSync()).to.equal(4);
    });

    it('List remove async', async () => {
        expect(await list!.remove(0)).to.equal('Some string');
        expect(await list!.size()).to.equal(3);
    });

    let list_cpy: ArrayListClass<JavaType> | null = null;

    it('List copy', () => {
        list_cpy = new ArrayList!(list!);

        expect(list!.sizeSync()).to.equal(list_cpy!.sizeSync());
    });

    it('List clear', () => {
        expect(list!.isEmptySync()).to.be.false;
        list!.clearSync();
        expect(list!.isEmptySync()).to.be.true;
        expect(list!.sizeSync()).to.equal(0);
    });

    it('List clear async', async () => {
        expect(await list_cpy!.isEmpty()).to.be.false;
        await list_cpy!.clear();
        expect(await list_cpy!.isEmpty()).to.be.true;
        expect(await list_cpy!.size()).to.equal(0);
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
    }).timeout(timeout);

    it('toString', async () => {
        const list = new ArrayList!();
        list.addSync('Hello');
        list.addSync('World');

        expect(list.toStringSync()).to.equal('[Hello, World]');
        expect(list.toString()).to.equal('[Hello, World]');
        expect(await list.toStringAsync()).to.equal('[Hello, World]');
        expect(list + '').to.equal('[Hello, World]');
    });

    const streamAsStringRegex =
        /java\.util\.stream\.ReferencePipeline\$Head@[0-9a-z]+/;

    it('stream toString', () => {
        const list = new ArrayList!();
        list.addSync('Hello');
        list.addSync('World');

        const stream = list.streamSync();
        expect(stream.toString()).to.match(streamAsStringRegex);
        expect(stream + '').to.match(streamAsStringRegex);
    });

    it('stream toString (async)', async () => {
        const list = await ArrayList!.newInstanceAsync();
        await list.add('Hello');
        await list.add('World');

        const stream = await list.stream();
        expect(await stream.toStringAsync()).to.match(streamAsStringRegex);
        expect(stream.toString()).to.match(streamAsStringRegex);
        expect(stream + '').to.match(streamAsStringRegex);
    });

    it('stream as list toString', () => {
        const list = new ArrayList!();
        list.addSync('Hello');
        list.addSync('World');

        const streamAsList = list.streamSync().toListSync();
        expect(streamAsList.toString()).to.equal('[Hello, World]');
        expect(streamAsList + '').to.equal('[Hello, World]');
    });

    it('stream as list toString (async)', async () => {
        const list = await ArrayList!.newInstanceAsync();
        await list.add('Hello');
        await list.add('World');

        const streamAsList = await (await list.stream()).toList();
        expect(await streamAsList.toStringAsync()).to.equal('[Hello, World]');
        expect(streamAsList.toString()).to.equal('[Hello, World]');
        expect(streamAsList + '').to.equal('[Hello, World]');
    });
});
