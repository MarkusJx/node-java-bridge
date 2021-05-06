import java, {java_instance_proxy} from "../index";
import assert = require("assert");

declare class JArrayList<T> extends java_instance_proxy {
    sizeSync(): number;

    addSync(data: T): void;

    getSync(index: number): T;
}

describe('ArrayListTest', () => {
    let ArrayList: typeof JArrayList = null;
    let list: JArrayList<number> = null;

    it('Create a new java instance', function () {
        java.ensureJVM();
    });

    it('Import java.util.ArrayList', () => {
        ArrayList = java.importClass('java.util.ArrayList') as typeof JArrayList;
    })

    it('Create a new ArrayList', () => {
        list = new ArrayList();
    });

    it('Get the list size', () => {
        assert.strictEqual(list.sizeSync(), 0);
    });

    it('Insert data', () => {
        list.addSync(123);
        assert.strictEqual(list.sizeSync(), 1);
    });

    it('Get data', () => {
        assert.strictEqual(list.getSync(0), 123);
    });
});
