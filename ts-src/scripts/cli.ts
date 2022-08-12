#!/usr/bin/env node
import yargs from 'yargs';
import { performance } from 'perf_hooks';
import path from 'path';

interface Args {
    classnames: string[];
    output: string;
    classpath?: string | string[];
}

type YargsHandler<T> = (args: yargs.ArgumentsCamelCase<T>) => Promise<void>;

const importOra = (): Promise<typeof import('ora').default> => eval("import('ora').then(ora => ora.default)");
const importChalk = (): Promise<typeof import('chalk').default> => eval("import('chalk').then(chalk => chalk.default)");

const builder: yargs.BuilderCallback<{}, Args> = (command) => {
    command.positional('classnames', {
        describe: 'The fully qualified class name(s) to convert',
        type: 'string',
    });

    command.positional('output', {
        describe: 'The output file',
        type: 'string',
    });

    command.option('classpath', {
        alias: 'cp',
        type: 'string',
        describe: 'The classpath to use',
    });
};

const handler: YargsHandler<Args> = async ({ classnames, output, classpath }) => {
    let destroyJVM: (() => void) | null = null;
    try {
        const startTime = performance.now();
        const java = await import('../.');
        destroyJVM = java.destroyJVM;

        if (classpath) {
            await java.classpath.appendAsync(classpath);
        }

        const chalk = await importChalk();
        const ora = await importOra();

        const version = await import(path.join(__dirname, '..', '..', 'package.json')).then(({ version }) => version);
        console.log(
            `Starting ${chalk.cyanBright('@markusjx/java')} ${chalk.greenBright(
                'v' + version
            )} Java definition generator`
        );

        java.ensureJVM();
        const javaInstance = java.getJavaInstance()!;
        const loadedJars = javaInstance.loadedJars;
        if (loadedJars.length > 0) {
            console.log(
                `Started JVM with version ${chalk.cyanBright(javaInstance.version)} and classpath '${loadedJars
                    .map((j) => chalk.cyanBright(j))
                    .join(';')}'`
            );
        }

        console.log(
            `Converting classes ${classnames
                .map((c) => chalk.magentaBright(c))
                .join(', ')} to typescript and saving result to ${chalk.cyanBright(path.normalize(output))}`
        );

        const spinner = ora().start();

        const resolvedImports: string[] = [];
        let resolvedCounter: number = 0;
        let numResolved: number = 0;

        let approximateTimeElapsed: number = 0;
        let lastClassResolved: string = '';
        const timeElapsedInterval = setInterval(() => {
            approximateTimeElapsed += 1;
            setText();
        }, 1000);

        const setText = () => {
            spinner.text = chalk.gray(
                `Elapsed time: ${chalk.yellow(approximateTimeElapsed)} seconds ${chalk.white(
                    '|'
                )} Converted ${chalk.cyanBright(resolvedCounter)} classes ${chalk.white(
                    '|'
                )} Converting class ${chalk.magentaBright(lastClassResolved)}`
            );
        };

        const TypescriptDefinitionGenerator = (await import('../TypescriptDefinitionGenerator')).default;

        for (const classname of classnames) {
            const generator = new TypescriptDefinitionGenerator(
                classname,
                (name) => {
                    lastClassResolved = name;
                    resolvedCounter++;
                    setText();
                },
                resolvedImports
            );
            const generated = await generator.generate();
            numResolved += generated.length;

            spinner.text = 'saving results';
            await TypescriptDefinitionGenerator.save(generated, output);
        }

        clearInterval(timeElapsedInterval);
        const timeElapsed = ((performance.now() - startTime) / 1000).toFixed(1);
        spinner.succeed(
            `Success - Converted ${chalk.blueBright(numResolved)} classes in ${chalk.blueBright(timeElapsed)} seconds`
        );
    } catch (e) {
        console.error(e);
        if (destroyJVM) destroyJVM();
        process.exit(1);
    }
};

yargs
    .command<Args>('* <output> <classnames..>', false, builder, handler)
    .parse();
