import { getJavaVersion, getJavaVersionSync, appendClasspath } from '../.';
import { it } from 'mocha';
import { expect } from 'chai';
import { shouldIncreaseTimeout } from './testUtil';

const timeout = shouldIncreaseTimeout ? 20e3 : 2e3;

describe('util test', () => {
    it('Get java version', async () => {
        const version = await getJavaVersion();
        expect(version).to.be.a('string').to.have.length.greaterThanOrEqual(1);
    }).timeout(timeout);

    it('Get java version (sync)', () => {
        const version = getJavaVersionSync();
        expect(version).to.be.a('string').to.have.length.greaterThanOrEqual(1);
    }).timeout(timeout);

    it('Add single jar to classpath', () => {
        appendClasspath('');
    }).timeout(timeout);
});
