import java, { JavaClassInstance } from '../.';

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

    it('Redirect stdout', (done) => {
        java.stdoutRedirect.enableRedirect((msg) => {
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
        java.stdoutRedirect.reset();
    });

    it('Redirect stderr', (done) => {
        java.stdoutRedirect.enableRedirect(null, (msg) => {
            if (msg === 'test') {
                done();
            } else {
                done(msg);
            }
        });

        const system = java.importClass('java.lang.System');
        system.err.printlnSync('test');
        system.err.flushSync();
    }).timeout(10000);

    it('reset', () => {
        java.stdoutRedirect.reset();
    });
});
