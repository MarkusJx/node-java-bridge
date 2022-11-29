import { importClass, importClassAsync, appendClasspath } from '../.';
import { expect } from 'chai';
import { ClassTool, shouldIncreaseTimeout } from './testUtil';
import path from 'path';

const timeout = shouldIncreaseTimeout ? 60e3 : 20e3;
let classTool: ClassTool | null = null;

function createJarWithBasicClass(
    pkgName: string,
    className: string,
    jarName: string
): void {
    classTool!.writeClass(
        `
        package ${pkgName};
        
        public class ${className} {
        
        }`,
        className
    );

    const fileName = `${pkgName.replaceAll('.', '/')}/${className}.class`;
    classTool!.createJar(jarName).addFile(fileName, fileName).close();
}

describe('ClassTest', () => {
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
        appendClasspath(path.join(classTool!.outDir, 'dir') + '/');

        const Class1 = importClass('dir.Class1');
        const Class2 = importClass('dir.Class2');

        expect(Class1).to.be.a('function');
        expect(Class2).to.be.a('function');

        const instance = new Class1();
        expect(instance).to.be.an('object');

        const instance2 = new Class2();
        expect(instance2).to.be.an('object');
    }).timeout(timeout);

    after(() => {
        classTool?.dispose();
    });
});
