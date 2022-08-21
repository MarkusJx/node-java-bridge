import java, { JavaClassInstance, JavaInterfaceProxy } from '../.';
import assert from 'assert';
import { expect } from 'chai';
import { afterEach } from 'mocha';
require('expose-gc');

declare class JThread extends JavaClassInstance {
    public constructor(proxy: JavaInterfaceProxy);

    public startSync(): void;

    public joinSync(): void;
}

describe('ProxyTest', () => {
    describe('java.lang.Runnable proxy', () => {
        it('Ensure jvm', () => {
            java.ensureJvm();
        });

        let thread: JThread | null = null;
        let proxy: JavaInterfaceProxy | null = null;

        it('Create a new proxy', (done) => {
            const Thread = java.importClass<typeof JThread>('java.lang.Thread');
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
            proxy!.reset();

            try {
                proxy!.reset();
                assert.fail('The proxy should already be destroyed');
            } catch (_) {}
        });

        after(() => {
            proxy = null;
            thread = null;
            global.gc!();
        });
    });

    describe('java.util.function.Function proxy', () => {
        let proxy: JavaInterfaceProxy | null = null;

        it('Create a new proxy', async () => {
            proxy = java.newProxy('java.util.function.Function', {
                apply: (arg: string): string => {
                    return arg.toUpperCase();
                },
            });

            const JString = java.importClass('java.lang.String');
            const str = new JString('hello');
            console.log(str);
            expect(await str.transform(proxy)).to.equal('HELLO');
        });

        it('Proxy with error', async () => {
            proxy = java.newProxy('java.util.function.Function', {
                apply: (): string => {
                    throw new Error('Error');
                },
            });

            const JString = java.importClass('java.lang.String');
            const str = new JString('hello');

            try {
                await str.transform(proxy);
                assert.fail('Should have thrown');
            } catch (e: any) {
                expect(e.message).to.contain(
                    'io.github.markusjx.bridge.JavascriptException: Error'
                );
            }
        });

        afterEach(() => {
            proxy!.reset();
        });

        after(() => {
            proxy = null;
            global.gc!();
        });
    });
});
