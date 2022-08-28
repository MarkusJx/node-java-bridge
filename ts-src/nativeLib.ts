import path from 'path';
import fs, { readFileSync } from 'fs';
import glob from 'glob';

const { platform, arch } = process;

function getModule(base: string): string {
    const local = path.join(__dirname, base + '.node');

    if (fs.existsSync(local)) {
        return local;
    } else {
        const module = '@markusjx/' + base.replaceAll('.', '-');
        // @ts-ignore
        if (__non_webpack_require__ && __non_webpack_require__.resolve) {
            // @ts-ignore
            return __non_webpack_require__.resolve(module);
        } else {
            return require.resolve(module);
        }
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

export function getNativeLibPath(): string {
    switch (platform) {
        case 'android':
            switch (arch) {
                case 'arm64':
                    return getModule('java.android-arm64');
                case 'arm':
                    return getModule('java.android-arm-eabi');
                default:
                    throw UnsupportedPlatform();
            }
        case 'win32':
            return getModule(`java.win32-${arch}-msvc`);
        case 'darwin':
            return getModule(`java.darwin-${arch}`);
        case 'freebsd':
            return getModule(`java.freebsd-${arch}`);
        case 'linux':
            switch (arch) {
                case 'x64':
                case 'arm64':
                    return getModule(
                        `java.linux-${arch}-${isMusl() ? 'musl' : 'gnu'}`
                    );
                case 'arm':
                    return getModule('java.linux-arm-gnueabihf');
                default:
                    throw UnsupportedPlatform();
            }
        default:
            throw UnsupportedPlatform();
    }
}

export function getJavaLibPath(): string {
    const dir = path.join(__dirname, '..', 'java-src', 'build', 'libs');
    let files = glob.sync('*.jar', { cwd: dir });

    if (files.length === 0) {
        throw new Error(`No java lib found in ${dir}`);
    } else {
        return path.join(dir, files[0]);
    }
}
