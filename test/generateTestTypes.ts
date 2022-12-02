import { TypescriptBulkDefinitionGenerator } from '../.';
import path from 'path';
import fs from 'fs';
import type { Ora } from 'ora';
import isCi from 'is-ci';
import type { ChalkInstance } from 'chalk';

interface Cache {
    classes: string[];
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

async function run() {
    if (fs.existsSync(cacheFile)) {
        try {
            const contents: Cache = JSON.parse(
                fs.readFileSync(cacheFile, 'utf8')
            );

            if (
                contents.classes &&
                arrayEquals(contents.classes, classesToGenerate)
            ) {
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

    gen.moduleDeclarations.forEach((declaration, i) => {
        gen.moduleDeclarations[i] = {
            ...declaration,
            contents: declaration.contents.replaceAll(
                'from "java-bridge";',
                `from ${JSON.stringify(path.join(__dirname, '..'))};`
            ),
        };
    });

    await gen.save(outDir);

    const cache: Cache = {
        classes: classesToGenerate,
    };
    fs.writeFileSync(cacheFile, JSON.stringify(cache, null, 4), 'utf8');
    spinner?.succeed('Generated Java definitions');
}

run().catch((err) => {
    spinner?.fail('Failed to convert classes');
    console.error(err);
    process.exit(1);
});
