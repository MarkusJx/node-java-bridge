import bench from 'nanobench';
import java, { JavaClassInstance } from '../../.';
import { ArrayList } from '../../.';

new ArrayList();

declare class StringClass extends JavaClassInstance {
    public static valueOfSync(value: string): String;
    public static valueOf(value: string): Promise<String>;
}

const iterations = 100000;

bench('Create JVM', (b) => {
    b.start();
    java.createJVM();
    b.end();
});

bench('Import class', (b) => {
    b.start();
    java.importClass('java.lang.String');
    b.end();
});

bench('Import cached class', (b) => {
    b.start();
    java.importClass('java.lang.String');
    b.end();
});

bench('Create strings', (b) => {
    const String = java.importClass<typeof StringClass>('java.lang.String');

    b.start();
    for (let i = 0; i < iterations; i++) {
        String.valueOfSync('Hello World');
    }
    b.end();
});

bench('Create strings async', async (b) => {
    const String = java.importClass<typeof StringClass>('java.lang.String');

    b.start();
    for (let i = 0; i < iterations; i++) {
        await String.valueOf('Hello World');
    }
    b.end();
});

bench('Create strings async with daemon threads', async (b) => {
    java.setConfig({ useDaemonThreads: true });
    const String = java.importClass<typeof StringClass>('java.lang.String');

    b.start();
    for (let i = 0; i < iterations; i++) {
        await String.valueOf('Hello World');
    }
    b.end();

    java.setConfig({ useDaemonThreads: false });
});
