import { TypescriptBulkDefinitionGenerator } from '../.';
import path from 'path';
import fs from 'fs';
import type { Ora } from 'ora';

const gen = new TypescriptBulkDefinitionGenerator();
const outDir = path.join(__dirname, 'javaDefinitions');

const importOra = (): Promise<typeof import('ora').default> =>
    eval("import('ora').then(ora => ora.default)");
const importChalk = (): Promise<typeof import('chalk').default> =>
    eval("import('chalk').then(chalk => chalk.default)");

let spinner: Ora | null = null;

async function run() {
    if (fs.existsSync(outDir)) {
        return;
    }

    const chalk = await importChalk();
    const ora = await importOra();
    spinner = ora('Generating Java definitions').start();

    await gen.generate(
        [
            'java.io.FileOutputStream',
            'java.util.jar.JarOutputStream',
            'java.util.jar.Manifest',
            'java.util.jar.Attributes',
            'java.io.File',
            'javax.tools.ToolProvider',
            'java.net.URLClassLoader',
            'java.util.jar.JarEntry',
            'java.io.FileInputStream',
        ],
        (classname) => {
            spinner.text = chalk.gray(
                `Converting class ${chalk.magentaBright(classname)}`
            );
        }
    );

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
    spinner.succeed('Generated Java definitions');
}

run().catch((err) => {
    spinner?.fail('Failed to convert classes');
    console.error(err);
    process.exit(1);
});
