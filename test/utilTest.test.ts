import { getJavaVersion, getJavaVersionSync } from '../.';
import { it } from 'mocha';
import { expect } from 'chai';

describe('util test', () => {
    it('Get java version', async () => {
        const version = await getJavaVersion();
        expect(version).to.be.a('string').to.have.length.greaterThanOrEqual(1);
    });

    it('Get java version (sync)', () => {
        const version = getJavaVersionSync();
        expect(version).to.be.a('string').to.have.length.greaterThanOrEqual(1);
    });
});
