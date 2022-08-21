import bench from 'nanobench';
import { ensureJvm, importClass, JavaClassInstance } from '../../ts-src';

const iterations = 100000;

declare class StringClass extends JavaClassInstance {
    public static valueOfSync(value: string): String;
    public static valueOf(value: string): Promise<String>;
}

bench('Create strings async with daemon threads', async (b) => {
    ensureJvm(null, null, null, { useDaemonThreads: true });
    const JString = importClass<typeof StringClass>('java.lang.String');

    b.start();
    for (let i = 0; i < iterations; i++) {
        await JString.valueOf('Hello World');
    }
    b.end();
});
