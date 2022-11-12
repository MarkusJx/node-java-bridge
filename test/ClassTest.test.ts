import fs from 'fs';
import {
    importClass,
    setClassLoader,
    getClassLoader,
    importClassAsync,
} from '../.';
import path from 'path';
import { expect } from 'chai';
import * as os from 'os';
import { shouldIncreaseTimeout } from './testUtil';

let outDir: string | null = null;
const timeout = shouldIncreaseTimeout ? 60e3 : 20e3;

function createClass(code: string, className: string): void {
    if (!outDir) {
        outDir = fs.mkdtempSync(path.join(os.tmpdir(), 'java'));
    }

    const classFile = path.join(outDir, className + '.java');
    fs.writeFileSync(classFile, code, { encoding: 'utf8' });
    const File = importClass('java.io.File');
    const root = new File(outDir);

    const ToolProvider = importClass('javax.tools.ToolProvider');
    const compiler = ToolProvider.getSystemJavaCompilerSync();
    compiler.runSync(null, null, null, [classFile]);

    const URLClassLoader = importClass('java.net.URLClassLoader');
    const prevClassLoader = getClassLoader();
    const classLoader = URLClassLoader.newInstanceSync(
        [root.toURISync().toURLSync()],
        prevClassLoader
    );

    setClassLoader(classLoader);
}

describe('ClassTest', () => {
    before(() => {
        createClass(
            `public class BasicClass {
            public static String test = "abc";
            
            public String s1;
            public Long l1;
            public long l2;
            public Long l3;
            public boolean b1;
            
            public BasicClass(String s1, Long l1, long l2, Long l3, boolean b1) {
                this.s1 = s1;
                this.l1 = l1;
                this.l2 = l2;
                this.l3 = l3;
                this.b1 = b1;
            }
            
            public BasicClass(String s1, Long l1) {
                this(s1, l1, 0, null, false);
            }
            
            public BasicClass(Long l1) {
                this(null, l1, 0, null, false);
            }
        }`,
            'BasicClass'
        );

        createClass(
            `
        public class ClassWithExplicitJavaTypes {
            public String s1;
            public Long l1;
            public long l2;
            public Long l3;
            public boolean b1;
            
            public ClassWithExplicitJavaTypes(String s1, Long l1, long l2, Long l3, boolean b1) {
                this.s1 = s1;
                this.l1 = l1;
                this.l2 = l2;
                this.l3 = l3;
                this.b1 = b1;
            }
            
            public ClassWithExplicitJavaTypes(String s1, Long l1) {
                this(s1, l1, 0, null, false);
            }
            
            public ClassWithExplicitJavaTypes(Long l1) {
                this(null, l1, 0, null, false);
            }
        }
        `,
            'ClassWithExplicitJavaTypes'
        );
    });

    it('Class with basic types', () => {
        const Test = importClass('BasicClass');
        const instance = new Test('s', 1, 2, 3, true);

        expect(instance.s1).to.equal('s');
        expect(instance.l1).to.equal(1n);
        expect(instance.l2).to.equal(2n);
        expect(instance.l3).to.equal(3n);
        expect(instance.b1).to.equal(true);

        const instance2 = new Test('s', 1);
        expect(instance2.s1).to.equal('s');
        expect(instance2.l1).to.equal(1n);
        expect(instance2.l2).to.equal(0n);
        expect(instance2.l3).to.equal(null);
        expect(instance2.b1).to.equal(false);

        const instance3 = new Test(1);
        expect(instance3.s1).to.equal(null);
        expect(instance3.l1).to.equal(1n);
        expect(instance3.l2).to.equal(0n);
        expect(instance3.l3).to.equal(null);
        expect(instance3.b1).to.equal(false);
    }).timeout(timeout);

    it('Class with basic types (async)', async () => {
        const Test = await importClassAsync('BasicClass');
        const instance = await Test.newInstanceAsync('s', 12, 23, 34, true);

        expect(instance.s1).to.equal('s');
        expect(instance.l1).to.equal(12n);
        expect(instance.l2).to.equal(23n);
        expect(instance.l3).to.equal(34n);
        expect(instance.b1).to.equal(true);

        const instance2 = await Test.newInstanceAsync('s', 12);
        expect(instance2.s1).to.equal('s');
        expect(instance2.l1).to.equal(12n);
        expect(instance2.l2).to.equal(0n);
        expect(instance2.l3).to.equal(null);
        expect(instance2.b1).to.equal(false);

        const instance3 = await Test.newInstanceAsync(12);
        expect(instance3.s1).to.equal(null);
        expect(instance3.l1).to.equal(12n);
        expect(instance3.l2).to.equal(0n);
        expect(instance3.l3).to.equal(null);
        expect(instance3.b1).to.equal(false);
    }).timeout(timeout);

    it('Class with explicit java types', async () => {
        const Test = importClass('ClassWithExplicitJavaTypes');
        const JLong = importClass('java.lang.Long');
        const JString = importClass('java.lang.String');
        const JBoolean = importClass('java.lang.Boolean');

        const instance = new Test(
            new JString('str'),
            new JLong(5),
            new JLong(6),
            new JLong(7),
            new JBoolean(true)
        );

        expect(instance.s1).to.equal('str');
        expect(instance.l1).to.equal(5n);
        expect(instance.l2).to.equal(6n);
        expect(instance.l3).to.equal(7n);
        expect(instance.b1).to.equal(true);

        const instance2 = new Test(new JString('str'), new JLong(5));
        expect(instance2.s1).to.equal('str');
        expect(instance2.l1).to.equal(5n);
        expect(instance2.l2).to.equal(0n);
        expect(instance2.l3).to.equal(null);
        expect(instance2.b1).to.equal(false);

        const instance3 = new Test(new JLong(5));
        expect(instance3.s1).to.equal(null);
        expect(instance3.l1).to.equal(5n);
        expect(instance3.l2).to.equal(0n);
        expect(instance3.l3).to.equal(null);
        expect(instance3.b1).to.equal(false);
    }).timeout(timeout);

    after(() => {
        if (outDir) {
            fs.rmSync(outDir, { recursive: true });
        }
    });
});
