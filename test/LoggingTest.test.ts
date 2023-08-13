import { expect } from 'chai';
import { logging } from '../.';
import path from 'path';

describe('LoggingTest', () => {
    it('check methods', () => {
        expect(logging.LOGGING_SUPPORTED).to.be.a('boolean');
        expect(logging.initLogger).to.be.a('function');
        expect(logging.setLogCallbacks).to.be.a('function');
        expect(logging.resetLogCallbacks).to.be.a('function');
    });

    it('check initLogger (logging disabled)', function () {
        if (logging.LOGGING_SUPPORTED) {
            this.skip();
        }

        expect(() => logging.initLogger('missing')).to.not.throw();
    });

    it('check initLogger (logging enabled)', function () {
        if (!logging.LOGGING_SUPPORTED) {
            this.skip();
        }

        expect(() =>
            logging.initLogger(path.join(__dirname, 'log.config.json'))
        ).to.not.throw();
        expect(() => logging.resetLogCallbacks()).to.not.throw();
    });

    it('check setLogCallbacks (logging disabled)', function () {
        if (logging.LOGGING_SUPPORTED) {
            this.skip();
        }

        expect(() =>
            logging.setLogCallbacks(
                () => {},
                () => {}
            )
        ).to.not.throw();
        expect(() => logging.resetLogCallbacks()).to.not.throw();
    });

    it('check setLogCallbacks (logging enabled)', async function () {
        if (!logging.LOGGING_SUPPORTED) {
            this.skip();
        }

        logging.initLogger(path.join(__dirname, 'log.config.json'));
        const promise = new Promise((resolve, reject) => {
            logging.setLogCallbacks(reject, resolve);
        });

        expect(await promise).to.match(
            /\d+-\d+-\d+T\d+:\d+:\d+.\d+\+\d+:\d+ \[.+] .+/gm
        );
        expect(() => logging.resetLogCallbacks()).to.not.throw();
    });
});
