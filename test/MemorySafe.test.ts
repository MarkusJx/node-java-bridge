import { expect } from 'chai';
import { importClass, ensureJvm } from '../.';

ensureJvm({
    opts: [
        '-Xms512m', 
        '-Xmx512m'
    ],
})

describe('MemorySafe tests', () => {
    it('should allow 1000 instances of java.io.ByteArrayInputStream', () => {
        const ByteArrayInputStream = importClass('java.io.ByteArrayInputStream');
        const buffer = Buffer.alloc(1024 * 1024);

        for (let i = 0; i < 200000; i++) {
            const stream = new ByteArrayInputStream(buffer);
            expect(stream).to.be.an('object');
        }
    })
})