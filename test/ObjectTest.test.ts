import { expect } from 'chai';
import { importClass } from '../.';

describe('Object test', () => {
    it('Create java.lang.Long from java.lang.String', () => {
        const JLong = importClass('java.lang.Long');
        const JString = importClass('java.lang.String');

        const l1 = new JString('12000');
        const l2 = new JLong(l1);
        expect(l2.longValueSync()).to.equal(12000n);
    });

    it('System.out', () => {
        const System = importClass('java.lang.System');

        expect(System.out).to.be.an('object');
        expect(() => (System.out = null)).to.throw();
    });
});
