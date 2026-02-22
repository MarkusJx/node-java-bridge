import { afterEach, beforeEach } from 'mocha';
import { clearClassProxies, config, importClass, importClassAsync } from '../.';
import { expect } from 'chai';
import { JString } from './classes';
import { checkJavaErrorCause } from './testUtil';

const SUFFIX_ERROR = 'syncSuffix and asyncSuffix cannot be the same';

describe('Config test', () => {
    beforeEach(() => {
        config.reset();
        clearClassProxies();
    });

    afterEach(() => {
        config.reset();
        clearClassProxies();
    });

    it('reset', () => {
        config.customInspect = true;
        config.runEventLoopWhenInterfaceProxyIsActive = true;

        expect(config.customInspect).to.be.true;
        expect(config.runEventLoopWhenInterfaceProxyIsActive).to.be.true;

        config.reset();

        expect(config.customInspect).to.be.false;
        expect(config.runEventLoopWhenInterfaceProxyIsActive).to.be.false;
    });

    it('custom inspect', () => {
        config.customInspect = true;

        expect(config.customInspect).to.be.true;

        const JavaString = importClass<typeof JString>('java.lang.String');
        const str = new JavaString('test');

        // @ts-ignore
        expect(str[Symbol.for('nodejs.util.inspect.custom')]()).to.equal(
            'test'
        );
    });

    it('set async suffix', async () => {
        config.asyncSuffix = 'Async';

        expect(config.asyncSuffix).to.equal('Async');

        const JavaString = importClass('java.lang.String');
        const str = new JavaString('test');

        expect(str.containsAsync).to.be.a('function');
        const res = str.containsAsync('e');
        expect(res).to.be.a('promise');
        expect(await res).to.equal(true);
    });

    it('set sync suffix', () => {
        config.syncSuffix = 'SyncSuffix';

        expect(config.syncSuffix).to.equal('SyncSuffix');

        const JavaString = importClass('java.lang.String');
        const str = new JavaString('test');

        expect(str.containsSyncSuffix).to.be.a('function');
        expect(str.containsSyncSuffix('e')).to.equal(true);
    });

    it('set invalid suffix', () => {
        expect(() => (config.asyncSuffix = 'Sync')).to.throw(SUFFIX_ERROR);

        config.asyncSuffix = null;
        expect(() => (config.syncSuffix = null)).to.throw(SUFFIX_ERROR);
        expect(() => (config.syncSuffix = '')).to.throw(SUFFIX_ERROR);

        config.asyncSuffix = 'Async';
        config.syncSuffix = '';

        expect(() => (config.asyncSuffix = '')).to.throw(SUFFIX_ERROR);
        expect(() => (config.asyncSuffix = null)).to.throw(SUFFIX_ERROR);
    });

    it('set complete config', async () => {
        config.config = {
            runEventLoopWhenInterfaceProxyIsActive: false,
            customInspect: true,
            asyncSuffix: 'Async',
            syncSuffix: 'Sync',
        };

        expect(config.runEventLoopWhenInterfaceProxyIsActive).to.be.false;
        expect(config.customInspect).to.be.true;
        expect(config.asyncSuffix).to.equal('Async');
        expect(config.syncSuffix).to.equal('Sync');

        const JavaString = importClass('java.lang.String');
        const str = new JavaString('test');

        expect(str.containsAsync).to.be.a('function');
        const res = str.containsAsync('e');
        expect(res).to.be.a('promise');
        expect(await res).to.equal(true);

        expect(str.containsSync).to.be.a('function');
        expect(str.containsSync('e')).to.equal(true);
    });

    it('set invalid config', () => {
        expect(() => {
            config.config = {
                runEventLoopWhenInterfaceProxyIsActive: false,
                customInspect: true,
                asyncSuffix: 'Sync',
                syncSuffix: 'Sync',
            };
        }).to.throw(SUFFIX_ERROR);

        expect(() => {
            config.config = {
                runEventLoopWhenInterfaceProxyIsActive: false,
                customInspect: true,
                asyncSuffix: '',
                syncSuffix: '',
            };
        }).to.throw(SUFFIX_ERROR);

        expect(() => {
            config.config = {
                runEventLoopWhenInterfaceProxyIsActive: false,
                customInspect: true,
                asyncSuffix: undefined,
                syncSuffix: undefined,
            };
        }).to.throw(SUFFIX_ERROR);
    });

    it('import class with config', async () => {
        const JavaString = importClass<typeof JString>('java.lang.String');
        const str = new JavaString('test');

        expect(str.toCharArray).to.be.a('function');
        expect(str.toCharArraySync).to.be.a('function');

        const res = str.toCharArray();
        expect(res).to.be.a('promise');
        expect(await res).to.deep.equal(['t', 'e', 's', 't']);
        expect(str.toCharArraySync()).to.deep.equal(['t', 'e', 's', 't']);

        const JavaStr = importClass('java.lang.String', {
            asyncSuffix: 'Async',
            syncSuffix: 'Sync',
        });

        const s = new JavaStr('test');

        expect(s.toCharArrayAsync).to.be.a('function');
        expect(s.toCharArraySync).to.be.a('function');

        const res2 = s.toCharArrayAsync();
        expect(res2).to.be.a('promise');
        expect(await res2).to.deep.equal(['t', 'e', 's', 't']);
        expect(s.toCharArraySync()).to.deep.equal(['t', 'e', 's', 't']);

        const JavaStr2 =
            await importClassAsync<typeof JString>('java.lang.String');
        const s2 = new JavaStr2('test');

        expect(s2.toCharArray).to.be.a('function');
        expect(s2.toCharArraySync).to.be.a('function');

        const res3 = s2.toCharArray();
        expect(res3).to.be.a('promise');
        expect(await res3).to.deep.equal(['t', 'e', 's', 't']);
        expect(s2.toCharArraySync()).to.deep.equal(['t', 'e', 's', 't']);
    });

    it('import class with config and invalid suffix', () => {
        expect(() =>
            importClass('java.lang.String', {
                asyncSuffix: 'Sync',
                syncSuffix: 'Sync',
            })
        ).to.throw(SUFFIX_ERROR);

        expect(() =>
            importClass('java.lang.String', {
                asyncSuffix: '',
                syncSuffix: '',
            })
        ).to.throw(SUFFIX_ERROR);
    });

    it('get java error in async context', async () => {
        const JavaString = importClass<typeof JString>('java.lang.String', {
            asyncJavaExceptionObjects: true,
        });

        try {
            await JavaString.newInstanceAsync(null);
            expect.fail('Expected an error');
        } catch (e) {
            checkJavaErrorCause(e, null);
        }

        try {
            await new JavaString('').split(null);
            expect.fail('Expected an error');
        } catch (e: unknown) {
            checkJavaErrorCause(
                e,
                'Cannot invoke "String.length()" because "regex" is null'
            );
        }

        try {
            await JavaString.valueOf(null);
            // May not throw an error
        } catch (e: unknown) {
            checkJavaErrorCause(
                e,
                'Cannot read the array length because "value" is null'
            );
        }
    });
});
