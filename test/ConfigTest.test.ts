import { afterEach, beforeEach } from 'mocha';
import { clearClassProxies, config, importClass } from '../.';
import { expect } from 'chai';

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

        const JString = importClass('java.lang.String');
        const str = new JString('test');

        // @ts-ignore
        expect(str[Symbol.for('nodejs.util.inspect.custom')]()).to.equal(
            'test'
        );
    });

    it('set async suffix', async () => {
        config.asyncSuffix = 'Async';

        expect(config.asyncSuffix).to.equal('Async');

        const JString = importClass('java.lang.String');
        const str = new JString('test');

        expect(str.containsAsync).to.be.a('function');
        const res = str.containsAsync('e');
        expect(res).to.be.a('promise');
        expect(await res).to.equal(true);
    });

    it('set sync suffix', () => {
        config.syncSuffix = 'SyncSuffix';

        expect(config.syncSuffix).to.equal('SyncSuffix');

        const JString = importClass('java.lang.String');
        const str = new JString('test');

        expect(str.containsSyncSuffix).to.be.a('function');
        expect(str.containsSyncSuffix('e')).to.equal(true);
    });

    it('set invalid suffix', () => {
        const error = 'syncSuffix and asyncSuffix cannot be the same';
        expect(() => (config.asyncSuffix = 'Sync')).to.throw(error);

        config.asyncSuffix = null;
        expect(() => (config.syncSuffix = null)).to.throw(error);
        expect(() => (config.syncSuffix = '')).to.throw(error);

        config.asyncSuffix = 'Async';
        config.syncSuffix = '';

        expect(() => (config.asyncSuffix = '')).to.throw(error);
        expect(() => (config.asyncSuffix = null)).to.throw(error);
    });
});
