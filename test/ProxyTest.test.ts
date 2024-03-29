import java, { importClass, JavaClass, JavaInterfaceProxy } from '../.';
import { expect, use } from 'chai';
import { afterEach } from 'mocha';
import semver from 'semver';
import { shouldIncreaseTimeout } from './testUtil';
import chaiAsPromised from 'chai-as-promised';

require('expose-gc');

use(chaiAsPromised);

declare class JThread extends JavaClass {
    public constructor(proxy: JavaInterfaceProxy);

    public startSync(): void;

    public joinSync(): void;

    public start(): Promise<void>;

    public join(): Promise<void>;
}

declare class JavaString extends JavaClass {
    public constructor(str: string);

    public transformSync(proxy: JavaInterfaceProxy): string;

    public transform(proxy: JavaInterfaceProxy): Promise<string>;
}

function getJavaVersion(): string {
    const version: string = importClass('java.lang.System')
        .getPropertySync('java.version')
        .split('_')[0]
        .replace(/^(\d+)-internal$/gm, '$1.0.0');

    const arr = version.split('.');
    arr.length = Math.min(arr.length, 3);
    return arr.join('.');
}

const javaVersion = getJavaVersion();
const timeoutMs: number = shouldIncreaseTimeout ? 60e3 : 2e3;

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

        it('Use destroyed proxy', () => {
            const proxy = java.newProxy('java.lang.Runnable', {
                run: () => {},
            });
            proxy.reset();

            expect(() => proxy.reset()).to.throw();
            const Thread = java.importClass<typeof JThread>('java.lang.Thread');
            expect(() => new Thread(proxy)).to.throw();
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

        before(() => {
            java.clearClassProxies();
            java.config.runEventLoopWhenInterfaceProxyIsActive = true;
        });

        it('Create a new proxy', async function () {
            if (shouldSkip) this.skip();

            proxy = java.newProxy('java.util.function.Function', {
                apply: (arg: string): string => {
                    return arg.toUpperCase();
                },
            });

            const JString =
                java.importClass<typeof JavaString>('java.lang.String');
            const str = new JString('hello');
            expect(await str.transform(proxy)).to.equal('HELLO');
        });

        it('Proxy (sync)', function () {
            if (shouldSkip) this.skip();

            proxy = java.newProxy('java.util.function.Function', {
                apply: (arg: string): string => {
                    return arg.toUpperCase();
                },
            });

            const JString =
                java.importClass<typeof JavaString>('java.lang.String');
            const str = new JString('hello');
            expect(str.transformSync(proxy)).to.equal('HELLO');
        });

        it('Proxy with error', async function () {
            if (shouldSkip) this.skip();
            proxy = java.newProxy('java.util.function.Function', {
                apply: (): string => {
                    throw new Error('Error');
                },
            });

            const JString =
                java.importClass<typeof JavaString>('java.lang.String');
            const str = new JString('hello');

            await expect(str.transform(proxy!)).to.eventually.be.rejectedWith(
                'io.github.markusjx.bridge.JavascriptException: Error'
            );
        });

        it('Proxy with error (sync)', function () {
            if (shouldSkip) this.skip();

            proxy = java.newProxy('java.util.function.Function', {
                apply: (): string => {
                    throw new Error('Error');
                },
            });

            const JString =
                java.importClass<typeof JavaString>('java.lang.String');
            const str = new JString('hello');

            expect(() => str.transformSync(proxy!)).to.throw(
                'io.github.markusjx.bridge.JavascriptException: Error'
            );
        });

        it('Proxy with async method', async function () {
            if (shouldSkip) this.skip();

            proxy = java.newProxy('java.util.function.Function', {
                apply: (arg: string): Promise<string> => {
                    return new Promise((resolve) => {
                        setTimeout(() => {
                            resolve(arg.toUpperCase());
                        }, 20);
                    });
                },
            });

            const JString =
                java.importClass<typeof JavaString>('java.lang.String');
            const str = new JString('hello');
            expect(await str.transform(proxy)).to.equal('HELLO');
        });

        it('Proxy with async error', async function () {
            if (shouldSkip) this.skip();

            proxy = java.newProxy('java.util.function.Function', {
                apply: (): Promise<never> => {
                    return new Promise((_, reject) => {
                        setTimeout(() => {
                            reject(new Error('Error'));
                        }, 20);
                    });
                },
            });

            const JString =
                java.importClass<typeof JavaString>('java.lang.String');
            const str = new JString('hello');

            await expect(str.transform(proxy!)).to.eventually.be.rejectedWith(
                'io.github.markusjx.bridge.JavascriptException: Error'
            );
        });

        it('Proxy with async method throwing error', async function () {
            if (shouldSkip) this.skip();

            proxy = java.newProxy('java.util.function.Function', {
                apply: async (): Promise<never> => {
                    throw new Error('Error');
                },
            });

            const JString =
                java.importClass<typeof JavaString>('java.lang.String');
            const str = new JString('hello');

            await expect(str.transform(proxy!)).to.eventually.be.rejectedWith(
                'io.github.markusjx.bridge.JavascriptException: Error'
            );
        });

        afterEach(() => {
            proxy?.reset();
        });

        after(function () {
            this.timeout(timeoutMs);
            proxy = null;
            global.gc!();
            java.config.runEventLoopWhenInterfaceProxyIsActive = false;
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

    describe('Daemon proxies', () => {
        const shouldSkip = semver.lt(javaVersion, '12.0.0');

        it('Scheduled proxy', (done) => {
            const proxy = java.newProxy(
                'java.lang.Runnable',
                {
                    run: () => {
                        done();
                    },
                },
                {
                    keepAsDaemon: true,
                }
            );

            (async () => {
                const TimeUnit = java.importClass(
                    'java.util.concurrent.TimeUnit'
                );
                const ScheduledThreadPoolExecutor = java.importClass(
                    'java.util.concurrent.ScheduledThreadPoolExecutor'
                );
                const executor =
                    await ScheduledThreadPoolExecutor.newInstanceAsync(1);
                await executor.schedule(proxy, 1, TimeUnit.SECONDS);
                await executor.shutdown();
                proxy.reset();

                expect(() => proxy.reset()).to.throw();
            })();
        }).timeout(timeoutMs * 10);

        it('Scheduled proxy (sync)', (done) => {
            const proxy = java.newProxy(
                'java.lang.Runnable',
                {
                    run: () => {
                        done();
                    },
                },
                {
                    keepAsDaemon: true,
                }
            );

            const TimeUnit = java.importClass('java.util.concurrent.TimeUnit');
            const ScheduledThreadPoolExecutor = java.importClass(
                'java.util.concurrent.ScheduledThreadPoolExecutor'
            );
            const executor = new ScheduledThreadPoolExecutor(1);
            executor.scheduleSync(proxy, 1, TimeUnit.SECONDS);
            executor.shutdownSync();
            proxy.reset();

            expect(() => proxy.reset()).to.throw();
        }).timeout(timeoutMs * 10);

        it('Basic proxy', async function () {
            if (shouldSkip) this.skip();

            let proxy = java.newProxy(
                'java.util.function.Function',
                {
                    apply: (arg: string): string => {
                        return arg.toUpperCase();
                    },
                },
                {
                    keepAsDaemon: true,
                }
            );

            const JString =
                java.importClass<typeof JavaString>('java.lang.String');
            const str = new JString('hello');
            expect(await str.transform(proxy)).to.equal('HELLO');
            proxy.reset();
        });

        it('Force reset', async function () {
            if (shouldSkip) this.skip();

            let proxy = java.newProxy(
                'java.util.function.Function',
                {
                    apply: (arg: string): string => {
                        return arg.toUpperCase();
                    },
                },
                {
                    keepAsDaemon: true,
                }
            );

            proxy.reset(true);
            expect(() => proxy.reset()).to.throw();

            const JString =
                java.importClass<typeof JavaString>('java.lang.String');
            const str = new JString('hello');
            expect(() => str.transformSync(proxy)).to.throw();
        });

        after(function () {
            this.timeout(timeoutMs);
            global.gc!();
            java.clearDaemonProxies();
        });
    });
});
