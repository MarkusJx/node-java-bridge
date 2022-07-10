import java, { JavaClassInstance, JavaInterfaceProxy } from '../.';
import assert from 'assert';
require('expose-gc');

declare class JThread extends JavaClassInstance {
    public constructor(proxy: JavaInterfaceProxy);

    public startSync(): void;

    public joinSync(): void;
}

describe('ProxyTest', () => {
    it('Ensure jvm', () => {
        java.ensureJVM();
    });

    let thread: JThread | null = null;
    let proxy: JavaInterfaceProxy | null = null;

    it('Create a new proxy', (done) => {
        const Thread = java.importClass('java.lang.Thread') as typeof JThread;
        proxy = java.newProxy('java.lang.Runnable', {
            run: () => {
                done();
            },
        });

        thread = new Thread(proxy);
        thread.startSync();
    });

    it('Join the thread', () => {
        thread!.joinSync();
    });

    it('Destroy the proxy', async () => {
        await proxy!.destroy();

        try {
            await proxy!.destroy();
            assert.fail('The proxy should already be destroyed');
        } catch (_) {}
    });

    after(() => {
        proxy = null;
        thread = null;
        global.gc!();
    });
});
