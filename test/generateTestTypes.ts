// @ts-nocheck
import { TypescriptBulkDefinitionGenerator, importClass } from '../.';
import path from 'path';
import fs from 'fs';
import type { Ora } from 'ora';
import isCi from 'is-ci';
import type { ChalkInstance } from 'chalk';
import { hashElement } from 'folder-hash';
import { glob as _glob } from 'glob';
import { promisify } from 'util';

const isSystemTest = process.argv.includes('--system-test');

interface Cache {
    classes: string[];
    distHash: string;
    os: string;
    javaVersion: string;
}

const gen = new TypescriptBulkDefinitionGenerator();
const outDir = path.join(__dirname, 'javaDefinitions');
const cacheFile = path.join(__dirname, 'javaDefinitions', 'cache.json');

const importOra = (): Promise<typeof import('ora').default> =>
    eval("import('ora').then(ora => ora.default)");
const importChalk = (): Promise<typeof import('chalk').default> =>
    eval("import('chalk').then(chalk => chalk.default)");

let spinner: Ora | null = null;

const classesToGenerate: string[] = [
    'java.io.FileOutputStream',
    'java.util.jar.JarOutputStream',
    'java.util.jar.Manifest',
    'java.util.jar.Attributes',
    'java.io.File',
    'javax.tools.ToolProvider',
    'java.net.URLClassLoader',
    'java.util.jar.JarEntry',
    'java.io.FileInputStream',
    'java.lang.System',
];

function arrayEquals<T>(a: T[], b: T[]) {
    return (
        Array.isArray(a) &&
        Array.isArray(b) &&
        a.length === b.length &&
        b.sort() &&
        a.sort().every((val, index) => val === b[index])
    );
}

function allEqual<T extends Record<string, any>>(
    expected: T,
    actual: T
): boolean {
    return Object.keys(expected).every((key) => {
        if (!actual) return false;

        if (Array.isArray(expected[key])) {
            return arrayEquals(expected[key], actual[key]);
        } else if (!!expected[key] && typeof expected[key] === 'object') {
            return allEqual(expected[key], actual[key]);
        } else {
            return expected[key] === actual[key];
        }
    });
}

async function calculateCache(): Promise<Cache> {
    const System = importClass('java.lang.System');
    const distDir = path.dirname(require.resolve('../.'));

    return {
        classes: classesToGenerate,
        distHash: (await hashElement(distDir)).hash,
        javaVersion: System.getPropertySync('java.version'),
        os: process.platform,
    };
}

async function run() {
    const newCache = await calculateCache();
    if (fs.existsSync(cacheFile)) {
        try {
            const contents: Cache = JSON.parse(
                fs.readFileSync(cacheFile, 'utf8')
            );

            if (allEqual(newCache, contents)) {
                return;
            }
        } catch (_) {}
    }

    let chalk: ChalkInstance | null = null;
    if (!isCi) {
        chalk = await importChalk();
        const ora = await importOra();
        spinner = ora('Generating Java definitions').start();
    }

    await gen.generate(classesToGenerate, (classname) => {
        if (spinner && chalk) {
            spinner.text = chalk.gray(
                `Converting class ${chalk.magentaBright(classname)}`
            );
        } else {
            console.log(`Converting '${classname}' to typescript`);
        }
    });

    if (spinner && chalk)
        spinner.text = chalk.gray('Writing definitions to disk');

    if (!isSystemTest) {
        gen.moduleDeclarations.forEach((declaration, i) => {
            gen.moduleDeclarations[i] = {
                ...declaration,
                contents: declaration.contents.replaceAll(
                    'from "java-bridge";',
                    `from ${JSON.stringify(path.join(__dirname, '..'))};`
                ),
            };
        });
    }

    await gen.save(outDir);

    fs.writeFileSync(cacheFile, JSON.stringify(newCache, null, 4), 'utf8');
    spinner?.succeed('Generated Java definitions');
}

async function updateImports(): Promise<void> {
    console.log('Updating imports');

    const glob = promisify(_glob);
    await Promise.all(
        (await glob('**/*.ts', { cwd: outDir }))
            .map((file) => path.join(outDir, file))
            .map(async (file) => {
                const contents = await fs.promises.readFile(file, 'utf8');
                await fs.promises.writeFile(
                    file,
                    contents.replaceAll(
                        'from "java-bridge";',
                        `from ${JSON.stringify(path.join(__dirname, '..'))};`
                    )
                );
            })
    );
}

if (process.argv.includes('--update-imports')) {
    updateImports().catch((err) => {
        console.error(err);
        process.exit(1);
    });
} else {
    run().catch((err) => {
        spinner?.fail('Failed to convert classes');
        console.error(err);
        process.exit(1);
    });
}
