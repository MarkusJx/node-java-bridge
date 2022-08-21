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

        const system = java.importClass('java.lang.System');
        system.err.printlnSync('test');
        system.err.flushSync();
    }).timeout(10000);

    it('reset', () => {
        redirect!.reset();
    });
});
