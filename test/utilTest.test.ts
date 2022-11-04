import { getJavaVersion, getJavaVersionSync } from '../.';
import { it } from 'mocha';
import { expect } from 'chai';
import isCI from 'is-ci';

let timeout = 2e3;
if (isCI && (process.arch === 'arm' || process.arch === 'arm64')) {
    timeout = 20e3;
}

describe('util test', () => {
    it('Get java version', async () => {
        const version = await getJavaVersion();
        expect(version).to.be.a('string').to.have.length.greaterThanOrEqual(1);
    }).timeout(timeout);

    it('Get java version (sync)', () => {
        const version = getJavaVersionSync();
        expect(version).to.be.a('string').to.have.length.greaterThanOrEqual(1);
    }).timeout(timeout);
});
