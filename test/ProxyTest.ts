import java, {java_function_caller_class, java_instance_proxy} from "../index";

declare class JThread extends java_instance_proxy {
    public constructor(proxy: java_function_caller_class);

    public startSync(): void;

    public joinSync(): void;
}

describe('ProxyTest', () => {
    it('Ensure jvm', () => {
        java.ensureJVM();
    });

    let thread: JThread = null;

    it('Create a new proxy', (done) => {
        const Thread = java.importClass('java.lang.Thread') as typeof JThread;
        const proxy = java.newProxy('java.lang.Runnable', {
            run: () => {
                done();
            }
        });

        thread = new Thread(proxy);
        thread.startSync();
    });

    it('Join the thread', () => {
        thread.joinSync();
    });
});