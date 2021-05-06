import java, {java_instance_proxy} from "../index";
import assert = require("assert");
import LogLevel = java.logging.LogLevel;

declare class JArrayList extends java_instance_proxy {
    sizeSync(): number;

    addSync(data: any): void;
}



describe('ArrayListTest', () => {
    let ArrayList: typeof JArrayList = null;
    let list: JArrayList = null;

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
    });
});
