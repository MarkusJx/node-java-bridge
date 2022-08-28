import java, { JavaClassInstance, StdoutRedirectGuard } from '../.';

interface Stream {
    printlnSync(msg: string): void;
    flushSync(): void;
}

declare class System extends JavaClassInstance {
    public static readonly out: Stream;
    public static readonly err: Stream;
}

describe('RedirectTest', () => {
    const system = java.importClass<typeof System>('java.lang.System');
    let redirect: StdoutRedirectGuard | null = null;

    it('Redirect stdout', (done) => {
        redirect = java.stdout.enableRedirect((_, msg) => {
            if (msg === 'test') {
                done();
            } else {
                done(msg);
            }
        });

        system.out.printlnSync('test');
        system.out.flushSync();
    }).timeout(10000);

    it('reset', () => {
        redirect!.reset();
        redirect = null;
    });

    it('Redirect stderr', (done) => {
        redirect = java.stdout.enableRedirect(null, (_, msg) => {
            if (msg === 'test') {
                done();
            } else {
                done(msg);
            }
        });

        system.err.printlnSync('test');
        system.err.flushSync();
    }).timeout(10000);

    it('Change the stdout redirect method', (done) => {
        redirect!.on('stdout', (_, msg) => {
            if (msg === 'abc') {
                done();
            } else {
                done(msg);
            }
        });

        system.out.printlnSync('abc');
        system.out.flushSync();
    }).timeout(10000);

    it('Change the stderr redirect method', (done) => {
        redirect!.on('stderr', (_, msg) => {
            if (msg === 'err') {
                done();
            } else {
                done(msg);
            }
        });

        system.err.printlnSync('err');
        system.err.flushSync();
    }).timeout(10000);

    it('reset', () => {
        redirect!.reset();
    });
});
