const java = require('../index');
const assert = require("assert");

describe('StringTest', () => {
    let JavaString;
    it('Import java.lang.String', () => {
        JavaString = java.importClass('java.lang.String');
    });

    let s1;
    it('Create string instance', () => {
        s1 = new JavaString("some text");

        assert.strictEqual(s1.toStringSync(), "some text");
    });

    it('String async match', async function () {
        assert.strictEqual(await s1.toString(), "some text");
    });

    it('String match', () => {
        assert.strictEqual(new JavaString("some text").equalsSync(s1), true);
    });
});