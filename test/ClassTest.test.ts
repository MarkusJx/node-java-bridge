import fs from 'fs';
import { importClass, setClassLoader, getClassLoader } from '../.';
import path from 'path';
import { expect } from 'chai';
import * as os from 'os';

let outDir: string | null = null;

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
    it('should create class', () => {
        createClass(
            `public class TestClass {
            public static String test = "abc";
            
            public String s1;
            public Long l1;
            public long l2;
            public Long l3;
            public boolean b1;
            
            public TestClass(String s1, Long l1, long l2, Long l3, boolean b1) {
                this.s1 = s1;
                this.l1 = l1;
                this.l2 = l2;
                this.l3 = l3;
                this.b1 = b1;
            }
        }`,
            'TestClass'
        );

        const Test = importClass('TestClass');
        const instance = new Test('s', 1, 2, 3, true);

        expect(instance.s1).to.equal('s');
        expect(instance.l1).to.equal(1n);
        expect(instance.l2).to.equal(2);
        expect(instance.l3).to.equal(3n);
        expect(instance.b1).to.equal(true);
    });

    after(() => {
        if (outDir) {
            fs.rmSync(outDir, { recursive: true });
        }
    });
});
