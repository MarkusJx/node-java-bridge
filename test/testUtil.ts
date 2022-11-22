import isCi from 'is-ci';
import fs from 'fs';
import path from 'path';
import os from 'os';
import { importClass, setClassLoader, getClassLoader } from '../.';
import type { UnknownJavaClass } from '../ts-src';
import ToolProvider, {ToolProviderClass} from "./javaDefinitions/javax/tools/ToolProvider";
import URLClassLoader from "./javaDefinitions/java/net/URLClassLoader";
import ClassLoader from "./javaDefinitions/java/lang/ClassLoader";
import File from './javaDefinitions/java/io/File';
import Manifest from "./javaDefinitions/java/util/jar/Manifest";
import FileOutputStream from "./javaDefinitions/java/io/FileOutputStream";
import JarOutputStream from "./javaDefinitions/java/util/jar/JarOutputStream";

export const shouldIncreaseTimeout =
    isCi && (process.arch === 'arm64' || process.arch === 'arm');

console.log('Process arch:', process.arch);
console.log('Process platform:', process.platform);

export class JarTool {
    private readonly outputStream: UnknownJavaClass;

    constructor(private readonly rootDir: string, outFile: string) {
        const manifest = new Manifest();
        this.outputStream = new JarOutputStream(new FileOutputStream(outFile) as any);
    }
}

export class ClassTool {
    private readonly outDir: string;

    public constructor() {
        this.outDir = fs.mkdtempSync(path.join(os.tmpdir(), 'java'));
    }

    public createClass(code: string, className: string): void {
        const classFile = path.join(this.outDir, className + '.java');
        fs.writeFileSync(classFile, code, { encoding: 'utf8' });
        const root = new File(this.outDir);

        const compiler = ToolProvider.getSystemJavaCompilerSync();
        // @ts-ignore
        compiler.runSync(null, null, null, [classFile]);

        //const URLClassLoader = importClass('java.net.URLClassLoader');
        const prevClassLoader = getClassLoader() as ClassLoader;
        const classLoader = URLClassLoader.newInstanceSync(
            [root.toURISync().toURLSync()],
            prevClassLoader
        );

        setClassLoader(classLoader);
    }

    public createJar(
        code: string,
        className: string,
        jarName: string
    ): JarTool {
        return null!;
    }

    public dispose(): void {
        fs.rmdirSync(this.outDir, { recursive: true });
    }
}
