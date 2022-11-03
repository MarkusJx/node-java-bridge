import java, { importClass, JavaClass, JavaInterfaceProxy } from '../.';
import assert from 'assert';
import { expect } from 'chai';
import { afterEach } from 'mocha';
import semver from 'semver';
import isCi from 'is-ci';
require('expose-gc');

declare class JThread extends JavaClass {
    public constructor(proxy: JavaInterfaceProxy);

    public startSync(): void;

    public joinSync(): void;

    public start(): Promise<void>;

    public join(): Promise<void>;
}

function getJavaVersion(): string {
    const version: string = importClass('java.lang.System')
        .getPropertySync('java.version')
        .split('_')[0];

    const arr = version.split('.');
    arr.length = Math.min(arr.length, 3);
    return arr.join('.');
}

const javaVersion = getJavaVersion();
let timeoutMs: number = 2e3;
if (isCi && (process.arch === 'arm64' || process.arch === 'arm')) {
    timeoutMs = 60e3;
}

describe('ProxyTest', () => {
    describe('java.lang.Runnable proxy', () => {
        it('Ensure jvm', () => {
            java.ensureJvm();
        });

        let thread: JThread | null = null;
        let proxy: JavaInterfaceProxy | null = null;

        it('Create a new proxy', (done) => {
            const Thread = importClass<typeof JThread>('java.lang.Thread');
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
            expect(() => proxy!.reset()).to.throw();
        });

        after(function () {
            this.timeout(timeoutMs);
            proxy = null;
            thread = null;
            global.gc!();
        });
    });

    describe('java.util.function.Function proxy', () => {
        const shouldSkip = semver.lt(javaVersion, '12.0.0');
        let proxy: JavaInterfaceProxy | null = null;

        it('Create a new proxy', async function () {
            if (shouldSkip) this.skip();

            proxy = java.newProxy('java.util.function.Function', {
                apply: (arg: string): string => {
                    return arg.toUpperCase();
                },
            });

            const JString = java.importClass('java.lang.String');
            const str = new JString('hello');
            expect(await str.transform(proxy)).to.equal('HELLO');
        });

        it('Proxy with error', async function () {
            if (shouldSkip) this.skip();
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
            proxy?.reset();
        });

        after(function () {
            this.timeout(timeoutMs);
            proxy = null;
            global.gc!();
        });
    });

    describe('Multiple proxies', () => {
        const proxies: JavaInterfaceProxy[] = [];

        it('Create Runnable proxy', () => {
            proxies.push(
                java.newProxy('java.lang.Runnable', {
                    run: () => {},
                })
            );
        });

        it('Create Function proxy', () => {
            proxies.push(
                java.newProxy('java.util.function.Function', {
                    apply: (arg: string): string => {
                        return arg.toUpperCase();
                    },
                })
            );
        });

        it('Use Runnable proxy', async () => {
            const Thread = java.importClass<typeof JThread>('java.lang.Thread');
            const thread = new Thread(proxies[0]);
            await thread.start();
            await thread.join();
        });

        it('Use Function proxy', async function () {
            if (semver.lt(javaVersion, '12.0.0')) {
                this.skip();
            }

            const JString = java.importClass('java.lang.String');
            const str = new JString('hello');
            expect(await str.transform(proxies[1])).to.equal('HELLO');
        });

        after(function () {
            this.timeout(timeoutMs);
            proxies.forEach((proxy) => proxy.reset());
            proxies.length = 0;
            global.gc!();
        });
    });
});
