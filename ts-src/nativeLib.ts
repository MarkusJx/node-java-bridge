import path from 'path';
import fs, { readFileSync } from 'fs';

const { platform, arch } = process;

const APP_ASAR_REGEX = /([\\/])app\.asar([\\/])/gim;
const APP_ASAR_UNPACKED = '$1app.asar.unpacked$2';

function getModule(base: string, isPackagedElectron: boolean): string {
    const local = path.join(__dirname, base + '.node');

    if (fs.existsSync(local)) {
        if (isPackagedElectron) {
            console.warn('Using local native module in packaged Electron app');
        }

        return local;
    } else {
        const module = base.replaceAll('.', '-').replace('java', 'java-bridge');

        let res: string;
        if (__non_webpack_require__ && __non_webpack_require__.resolve) {
            res = __non_webpack_require__.resolve(module);
        } else {
            res = require.resolve(module);
        }

        if (isPackagedElectron)
            res = res.replace(APP_ASAR_REGEX, APP_ASAR_UNPACKED);
        return res;
    }
}

function UnsupportedPlatform(): Error {
    return new Error(`Unsupported platform: ${platform} ${arch}`);
}

function isMusl() {
    // For Node 10
    if (!process.report || typeof process.report.getReport !== 'function') {
        try {
            return readFileSync('/usr/bin/ldd', 'utf8').includes('musl');
        } catch (e) {
            return true;
        }
    } else {
        const { glibcVersionRuntime } = (process.report.getReport() as any)
            .header;
        return !glibcVersionRuntime;
    }
}

export function getNativeLibPath(isPackagedElectron: boolean): string {
    switch (platform) {
        case 'android':
            switch (arch) {
                case 'arm64':
                    return getModule('java.android-arm64', isPackagedElectron);
                case 'arm':
                    return getModule(
                        'java.android-arm-eabi',
                        isPackagedElectron
                    );
                default:
                    throw UnsupportedPlatform();
            }
        case 'win32':
            return getModule(`java.win32-${arch}-msvc`, isPackagedElectron);
        case 'darwin':
            return getModule(`java.darwin-${arch}`, isPackagedElectron);
        case 'freebsd':
            return getModule(`java.freebsd-${arch}`, isPackagedElectron);
        case 'linux':
            switch (arch) {
                case 'x64':
                case 'arm64':
                    return getModule(
                        `java.linux-${arch}-${isMusl() ? 'musl' : 'gnu'}`,
                        isPackagedElectron
                    );
                case 'arm':
                    return getModule(
                        'java.linux-arm-gnueabihf',
                        isPackagedElectron
                    );
                default:
                    throw UnsupportedPlatform();
            }
        default:
            throw UnsupportedPlatform();
    }
}

export function getJavaLibPath(): string {
    const lib = path.join(__dirname, 'JavaBridge.jar');

    if (fs.existsSync(lib) && fs.statSync(lib).isFile()) {
        return lib;
    } else {
        throw new Error('JavaBridge.jar not found');
    }
}
