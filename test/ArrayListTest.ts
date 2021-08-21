import java, {JavaType} from "../index";
import List from "../types/List";
import assert = require("assert");
import {it} from "mocha";

describe('ArrayListTest', () => {
    let ArrayList: typeof List = null;
    let list: List<JavaType> = null;

    it('Ensure jvm', () => {
        java.ensureJVM();
    }).timeout(10000);

    it('Import java.util.ArrayList', () => {
        ArrayList = java.importClass('java.util.ArrayList') as typeof List;
    })

    it('Create a new ArrayList', () => {
        list = new ArrayList();
        assert.notStrictEqual(list, null);
        assert.strictEqual(list.isEmptySync(), true);
    });

    it('Get the list size', () => {
        assert.strictEqual(list.sizeSync(), 0);
    });

    const elements = [];

    it('Insert int', () => {
        list.addSync(123);
        elements.push(123);

        assert.strictEqual(list.containsSync(123), true);
        assert.strictEqual(list.sizeSync(), 1);
        assert.strictEqual(list.getSync(0), 123);
    });

    it('Insert string', () => {
        const val = "Some string";
        list.addSync(val);
        elements.push(val);

        assert.strictEqual(list.lastIndexOfSync(val), 1);
        assert.strictEqual(list.sizeSync(), 2);
        assert.strictEqual(list.getSync(1), val);
    });

    it('Insert long', () => {
        let long = BigInt(2313213213);
        list.addSync(long);
        elements.push(long);

        assert.strictEqual(list.sizeSync(), 3);
        assert.strictEqual(list.getSync(2), long);
    });

    it('Insert double async', async () => {
        const double = 12.21341;
        await list.add(double);
        elements.push(double);

        assert.strictEqual(await list.lastIndexOf(double), 3);
        assert.strictEqual(await list.contains(double), true);
        assert.strictEqual(await list.isEmpty(), false);
        assert.strictEqual(await list.size(), 4);
        assert.strictEqual(await list.get(3), double);
    });

    it('Insert Float', () => {
        const Float = java.importClass('java.lang.Float');
        const val = new Float(1232.248);

        list.addSync(val);
        elements.push(val.doubleValueSync());
    });

    it('toArray', () => {
        // Create a conversion helper as json
        // doesn't know how to serialize a BigInt
        const converter = (key, value) => {
            if (typeof value === 'bigint') {
                return value.toString();
            } else {
                return value;
            }
        }

        assert.strictEqual(JSON.stringify(list.toArraySync(), converter), JSON.stringify(elements, converter));
    });

    it('List remove', () => {
        assert.strictEqual(list.removeSync(0), 123);
        assert.strictEqual(list.sizeSync(), 4);
    });

    it('List remove async', async () => {
        assert.strictEqual(await list.remove(0), "Some string");
        assert.strictEqual(await list.size(), 3);
    });

    let list_cpy: List<JavaType> = null;

    it('List copy', () => {
        list_cpy = new ArrayList(list);

        assert.strictEqual(list.sizeSync(), list_cpy.sizeSync());
    });

    it('List clear', () => {
        list.clearSync();
        assert.strictEqual(list.isEmptySync(), true);
        assert.strictEqual(list.sizeSync(), 0);
    });

    it('List clear async', async () => {
        await list_cpy.clear();
        assert.strictEqual(await list_cpy.isEmpty(), true);
        assert.strictEqual(await list_cpy.size(), 0);
    });
});
