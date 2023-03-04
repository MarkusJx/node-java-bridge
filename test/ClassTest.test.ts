import { importClass, importClassAsync, appendClasspath } from '../.';
import { expect } from 'chai';
import { ClassTool, shouldIncreaseTimeout } from './testUtil';
import path from 'path';

const shouldSkip = process.env.SKIP_CLASS_TEST === 'true';
const timeout = shouldIncreaseTimeout ? 60e3 : 20e3;
let classTool: ClassTool | null = null;

interface JarClassOpts {
    extraImports?: string[];
    extraCode?: string;
    classpath?: string[];
}

function createJarWithBasicClass(
    pkgName: string,
    className: string,
    jarName: string,
    opts?: JarClassOpts
): void {
    classTool!.writeClass(
        `
        package ${pkgName};
        ${opts?.extraImports?.map((i) => `import ${i};`)?.join('\n') ?? ''}
        
        public class ${className} {
            ${opts?.extraCode ?? ''}
        }`,
        className,
        opts?.classpath ?? []
    );

    const fileName = `${pkgName.replaceAll('.', '/')}/${className}.class`;
    classTool!.createJar(jarName).addFile(fileName, fileName).close();
}

describe('ClassTest', () => {
    if (shouldSkip) return;

    before(function () {
        this.timeout(timeout);
        if (!classTool) classTool = new ClassTool();

        classTool.createClass(
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

        classTool.createClass(
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

        classTool.createClass(
            `
        public class ClassWithProperties {
            public static String test = "test";
            public static final String test2 = "test2";
            public String s1 = "abc";
            public final String s2 = "def";
        }
            `,
            'ClassWithProperties'
        );

        classTool.createClass(
            `
        public class ClassWithComplexProperties {
            public static class InnerClass {
                public String s1 = "abc";
            }
            
            public InnerClass innerClass;
            public static InnerClass staticInnerClass = new InnerClass();
            
            public ClassWithComplexProperties() {
                this.innerClass = new InnerClass();
            }
        }
            `,
            'ClassWithComplexProperties'
        );

        createJarWithBasicClass('test', 'ClassWithPackage', 'first.jar');
        createJarWithBasicClass(
            'test',
            'ClassWithPackageAndImport',
            'second.jar'
        );
        createJarWithBasicClass('async', 'Class1', 'third.jar');
        createJarWithBasicClass('async', 'Class2', 'fourth.jar');
        createJarWithBasicClass('dir', 'Class1', 'dir/fifth.jar');
        createJarWithBasicClass('dir', 'Class2', 'dir/sixth.jar');
        createJarWithBasicClass('any', 'Class1', 'any/seventh.jar');
        createJarWithBasicClass('any', 'Class2', 'any/eighth.jar');
        createJarWithBasicClass('other', 'Class1', 'ninth.jar');
        createJarWithBasicClass('dir1', 'Class1', 'dir1/tenth.jar');
        createJarWithBasicClass('dir1', 'Class2', 'dir1/eleventh.jar');
        createJarWithBasicClass('external', 'ExternalClass', 'twelfth.jar');
        createJarWithBasicClass('importing', 'Class2', 'thirteenth.jar', {
            extraImports: ['external.ExternalClass'],
            extraCode: `
            private final ExternalClass ext;

            public Class2() {
                this.ext = new ExternalClass();
            }

            public ExternalClass getExt() {
                return this.ext;
            }
            `,
            classpath: [path.join(classTool!.outDir, 'twelfth.jar')],
        });
        createJarWithBasicClass('dyn.external', 'Class1', 'fourteenth.jar');
        createJarWithBasicClass('dyn.importing', 'Class2', 'fifteenth.jar', {
            extraImports: ['dyn.external.Class1'],
            extraCode: `
            public final Class<?> ext;

            public Class2() throws ClassNotFoundException {
                this.ext = Class.forName("dyn.external.Class1");
            }
            `,
            classpath: [path.join(classTool!.outDir, 'fourteenth.jar')],
        });
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

    it('Class with explicit java types', () => {
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

    it('Class with explicit java types (async)', async () => {
        const Test = await importClassAsync('ClassWithExplicitJavaTypes');
        const JLong = await importClassAsync('java.lang.Long');
        const JString = await importClassAsync('java.lang.String');
        const JBoolean = await importClassAsync('java.lang.Boolean');

        const instance = await Test.newInstanceAsync(
            await JString.newInstanceAsync('string'),
            await JLong.newInstanceAsync(51),
            await JLong.newInstanceAsync(61),
            await JLong.newInstanceAsync(71),
            await JBoolean.newInstanceAsync(true)
        );

        expect(instance.s1).to.equal('string');
        expect(instance.l1).to.equal(51n);
        expect(instance.l2).to.equal(61n);
        expect(instance.l3).to.equal(71n);
        expect(instance.b1).to.equal(true);

        const instance2 = await Test.newInstanceAsync(
            await JString.newInstanceAsync('string'),
            await JLong.newInstanceAsync(52)
        );
        expect(instance2.s1).to.equal('string');
        expect(instance2.l1).to.equal(52n);
        expect(instance2.l2).to.equal(0n);
        expect(instance2.l3).to.equal(null);
        expect(instance2.b1).to.equal(false);

        const instance3 = await Test.newInstanceAsync(
            await JLong.newInstanceAsync(53)
        );
        expect(instance3.s1).to.equal(null);
        expect(instance3.l1).to.equal(53n);
        expect(instance3.l2).to.equal(0n);
        expect(instance3.l3).to.equal(null);
        expect(instance3.b1).to.equal(false);
    }).timeout(timeout);

    it('Classes from multiple jars', () => {
        expect(() => importClass('test.ClassWithPackage')).to.throw();
        expect(() => importClass('test.ClassWithPackageAndImport')).to.throw();

        appendClasspath([
            path.join(classTool!.outDir, 'first.jar'),
            path.join(classTool!.outDir, 'second.jar'),
        ]);

        const ClassWithPackage = importClass('test.ClassWithPackage');
        const ClassWithPackageAndImport = importClass(
            'test.ClassWithPackageAndImport'
        );

        expect(ClassWithPackage).to.be.a('function');
        expect(ClassWithPackageAndImport).to.be.a('function');

        const instance = new ClassWithPackage();
        expect(instance).to.be.an('object');

        const instance2 = new ClassWithPackageAndImport();
        expect(instance2).to.be.an('object');
    }).timeout(timeout);

    it('Classes from multiple jars (async)', async () => {
        expect(() => importClass('async.Class1')).to.throw();
        expect(() => importClass('async.Class2')).to.throw();

        appendClasspath([
            path.join(classTool!.outDir, 'third.jar'),
            path.join(classTool!.outDir, 'fourth.jar'),
        ]);

        const Class1 = await importClassAsync('async.Class1');
        const Class2 = await importClassAsync('async.Class2');

        expect(Class1).to.be.a('function');
        expect(Class2).to.be.a('function');

        const instance = await Class1.newInstanceAsync();
        expect(instance instanceof Promise).to.be.false;
        expect(instance).to.be.an('object');

        const instance2 = await Class2.newInstanceAsync();
        expect(instance2 instanceof Promise).to.be.false;
        expect(instance2).to.be.an('object');
    }).timeout(timeout);

    it('Classes from multiple jars with directory import', () => {
        expect(() => importClass('dir.Class1')).to.throw();
        expect(() => importClass('dir.Class2')).to.throw();

        appendClasspath(path.join(classTool!.outDir, 'dir', '**/*'));

        const Class1 = importClass('dir.Class1');
        const Class2 = importClass('dir.Class2');

        expect(Class1).to.be.a('function');
        expect(Class2).to.be.a('function');

        const instance = new Class1();
        expect(instance).to.be.an('object');

        const instance2 = new Class2();
        expect(instance2).to.be.an('object');
    }).timeout(timeout);

    it('Classes from multiple jars with any import', () => {
        expect(() => importClass('any.Class1')).to.throw();
        expect(() => importClass('any.Class2')).to.throw();
        expect(() => importClass('other.Class1')).to.throw();

        appendClasspath([
            path.join(classTool!.outDir, 'any', '*'),
            path.join(classTool!.outDir, 'ninth.jar'),
        ]);

        const Class1 = importClass('any.Class1');
        const Class2 = importClass('any.Class2');
        const Class3 = importClass('other.Class1');

        expect(Class1).to.be.a('function');
        expect(Class2).to.be.a('function');
        expect(Class3).to.be.a('function');

        const instance = new Class1();
        expect(instance).to.be.an('object');

        const instance2 = new Class2();
        expect(instance2).to.be.an('object');

        const instance3 = new Class3();
        expect(instance3).to.be.an('object');
    }).timeout(timeout);

    it('Classes from multiple jars with dir import', () => {
        expect(() => importClass('dir1.Class1')).to.throw();
        expect(() => importClass('dir1.Class2')).to.throw();

        appendClasspath(path.join(classTool!.outDir, 'dir1', '*'));

        const Class1 = importClass('dir1.Class1');
        const Class2 = importClass('dir1.Class2');

        expect(Class1).to.be.a('function');
        expect(Class2).to.be.a('function');

        const instance = new Class1();
        expect(instance).to.be.an('object');

        const instance2 = new Class2();
        expect(instance2).to.be.an('object');
    }).timeout(timeout);

    it('Class with external dependency', () => {
        expect(() => importClass('importing.Class2')).to.throw();
        expect(() => importClass('external.ExternalClass')).to.throw();

        appendClasspath([
            path.join(classTool!.outDir, 'twelfth.jar'),
            path.join(classTool!.outDir, 'thirteenth.jar'),
        ]);

        const Class2 = importClass('importing.Class2');
        expect(Class2).to.be.a('function');

        const instance = new Class2();
        expect(instance).to.be.an('object');

        const ext = instance.getExtSync();
        expect(ext).to.be.an('object');
    }).timeout(timeout);

    it('Class with external dependency loaded dynamically', () => {
        expect(() => importClass('dyn.importing.Class2')).to.throw();
        expect(() => importClass('dyn.external.Class1')).to.throw();

        appendClasspath([
            path.join(classTool!.outDir, 'fourteenth.jar'),
            path.join(classTool!.outDir, 'fifteenth.jar'),
        ]);

        const Class2 = importClass('dyn.importing.Class2');
        expect(Class2).to.be.a('function');

        const instance = new Class2();
        expect(instance).to.be.an('object');

        const ext = instance.ext;
        expect(ext).to.be.an('object');

        const instance2 = ext.newInstanceSync();
        expect(instance2).to.be.an('object');
    }).timeout(timeout);

    it('Class with properties', () => {
        const Class = importClass('ClassWithProperties');

        expect(Class).to.be.a('function');
        expect(Class.test).to.be.a('string');
        expect(Class.test).to.equal('test');
        expect(() => (Class.test = 't')).to.not.throw();
        expect(Class.test).to.equal('t');

        expect(Class.test2).to.be.a('string');
        expect(Class.test2).to.equal('test2');
        expect(() => (Class.test2 = 't2')).to.throw();

        const instance = new Class();
        expect(instance).to.be.an('object');
        expect(instance.s1).to.be.a('string');
        expect(instance.s1).to.equal('abc');
        expect(() => (instance.s1 = 'def')).to.not.throw();
        expect(instance.s1).to.equal('def');

        expect(instance.s2).to.be.a('string');
        expect(instance.s2).to.equal('def');
        expect(() => (instance.s2 = 'def')).to.throw();
    }).timeout(timeout);

    it('Class with complex properties', () => {
        const Class = importClass('ClassWithComplexProperties');

        expect(Class).to.be.a('function');
        expect(Class.staticInnerClass).to.be.an('object');
        expect(Class.staticInnerClass).to.have.property('s1');
        expect(Class.staticInnerClass.s1).to.be.a('string');
        expect(Class.staticInnerClass.s1).to.equal('abc');

        const instance = new Class();
        expect(instance).to.be.an('object');
        expect(instance.innerClass).to.be.an('object');
        expect(instance.innerClass).to.have.property('s1');
        expect(instance.innerClass.s1).to.be.a('string');
        expect(instance.innerClass.s1).to.equal('abc');
    }).timeout(timeout);

    after(function () {
        this.timeout(timeout);
        try {
            classTool?.dispose();
        } catch (_) {}
    });
});
