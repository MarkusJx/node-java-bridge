import bench from 'nanobench';
import { JavaClassInstance, ensureJvm, importClass } from '../../.';

declare class StringClass extends JavaClassInstance {
    public static valueOfSync(value: string): String;
    public static valueOf(value: string): Promise<String>;
}

const iterations = 100000;

bench('Create JVM', (b) => {
    b.start();
    ensureJvm();
    b.end();
});

bench('Import class', (b) => {
    b.start();
    importClass('java.lang.String');
    b.end();
});

bench('Import cached class', (b) => {
    b.start();
    importClass('java.lang.String');
    b.end();
});

bench('Create strings', (b) => {
    const JString = importClass<typeof StringClass>('java.lang.String');

    b.start();
    for (let i = 0; i < iterations; i++) {
        JString.valueOfSync('Hello World');
    }
    b.end();
});

bench('Create strings async', async (b) => {
    const JString = importClass<typeof StringClass>('java.lang.String');

    b.start();
    for (let i = 0; i < iterations; i++) {
        await JString.valueOf('Hello World');
    }
    b.end();
});

bench('String constructors', (b) => {
    const JString = importClass<typeof StringClass>('java.lang.String');

    b.start();
    for (let i = 0; i < iterations; i++) {
        new JString('Hello World');
    }
    b.end();
});

bench('String constructors async', async (b) => {
    const JString = importClass<typeof StringClass>('java.lang.String');

    b.start();
    for (let i = 0; i < iterations; i++) {
        await JString.newInstanceAsync('Hello World');
    }
    b.end();
});
