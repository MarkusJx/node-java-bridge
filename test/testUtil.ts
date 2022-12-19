import isCi from 'is-ci';
import fs from 'fs';
import path from 'path';
import os from 'os';
import { setClassLoader, getClassLoader } from '../.';
import ToolProvider from './javaDefinitions/javax/tools/ToolProvider';
import URLClassLoader from './javaDefinitions/java/net/URLClassLoader';
import ClassLoader from './javaDefinitions/java/lang/ClassLoader';
import File from './javaDefinitions/java/io/File';
import Manifest from './javaDefinitions/java/util/jar/Manifest';
import FileOutputStream from './javaDefinitions/java/io/FileOutputStream';
import JarOutputStream from './javaDefinitions/java/util/jar/JarOutputStream';
import Attributes$Name from './javaDefinitions/java/util/jar/Attributes$Name';
import JarEntry from './javaDefinitions/java/util/jar/JarEntry';
import FileInputStream from './javaDefinitions/java/io/FileInputStream';
import System from './javaDefinitions/java/lang/System';

export const shouldIncreaseTimeout =
    isCi && (process.arch === 'arm64' || process.arch === 'arm');

console.log('Process arch:', process.arch);
console.log('Process platform:', process.platform);

export class JarTool {
    private readonly outputStream: JarOutputStream;

    constructor(private readonly rootDir: string, outFile: string) {
        const manifest = new Manifest();
        manifest
            .getMainAttributesSync()!
            .putSync(Attributes$Name.MANIFEST_VERSION, '1.0');
        this.outputStream = new JarOutputStream(
            new FileOutputStream(path.join(rootDir, outFile)),
            manifest
        );
    }

    public addFile(src: string, dest: string): this {
        const entry = new JarEntry(dest);
        const source = path.join(this.rootDir, src);
        entry.setTimeSync(new File(source).lastModifiedSync());
        this.outputStream.putNextEntrySync(entry);

        const inputStream = new FileInputStream(source);
        const data = inputStream.readAllBytesSync();
        inputStream.closeSync();
        this.outputStream.writeSync(data);
        this.outputStream.closeEntrySync();
        return this;
    }

    public close(): void {
        this.outputStream.closeSync();
    }
}

export class ClassTool {
    public readonly outDir: string;

    public constructor() {
        this.outDir = fs.mkdtempSync(path.join(os.tmpdir(), 'java'));
    }

    public writeClass(
        code: string,
        className: string,
        classpath: string[] = [],
        useGeneratedDir: boolean = false
    ): void {
        const outDir = useGeneratedDir
            ? path.join(this.outDir, 'generated')
            : this.outDir;

        if (useGeneratedDir && !fs.existsSync(outDir)) {
            fs.mkdirSync(outDir, {
                recursive: true,
            });
        }

        const classFile = path.join(outDir, className + '.java');
        fs.writeFileSync(classFile, code, { encoding: 'utf8' });

        const extraOpts: string[] = [];
        if (classpath.length > 0) {
            classpath.push(System.getPropertySync('java.class.path') || '.');
            extraOpts.push('-classpath', classpath.join(File.pathSeparator!));
        }

        const compiler = ToolProvider.getSystemJavaCompilerSync();
        const res = compiler!.runSync(null, null, null, [
            ...extraOpts,
            classFile,
            '-d',
            outDir,
        ]);

        if (res != 0) {
            throw new Error(`The compiler returned non-zero exit code: ${res}`);
        }
    }

    public createClass(
        code: string,
        className: string,
        classpath: string[] = []
    ): void {
        this.writeClass(code, className, classpath, true);
        const root = new File(path.join(this.outDir, 'generated'));

        const prevClassLoader = getClassLoader() as ClassLoader;
        const classLoader = URLClassLoader.newInstanceSync(
            [root.toURISync()!.toURLSync()],
            prevClassLoader
        );

        setClassLoader(classLoader!);
    }

    public createJar(jarName: string): JarTool {
        return new JarTool(this.outDir, jarName);
    }

    public dispose(): void {
        fs.rmSync(this.outDir, { recursive: true, force: true });
    }
}
