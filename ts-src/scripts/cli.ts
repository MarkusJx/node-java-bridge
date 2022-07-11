#!/usr/bin/env node
import yargs from 'yargs';
import { performance } from 'perf_hooks';
import path from 'path';

interface Args {
    classnames: string[];
    output: string;
}

const importOra = (): Promise<typeof import('ora').default> => eval("import('ora').then(ora => ora.default)");
const importChalk = (): Promise<typeof import('chalk').default> => eval("import('chalk').then(chalk => chalk.default)");

yargs
    .command<Args>(
        '* <output> <classnames..>',
        false,
        (command) => {
            command.positional('classnames', {
                describe: 'The fully qualified class name(s) to convert',
                type: 'string',
            });

            command.positional('output', {
                describe: 'The output file',
                type: 'string',
            });
        },
        async ({ classnames, output }) => {
            let destroyJVM: (() => void) | null = null;
            try {
                const startTime = performance.now();
                destroyJVM = await import('../.').then(({ destroyJVM }) => destroyJVM);

                const chalk = await importChalk();
                const ora = await importOra();

                const version = await import(path.join(__dirname, '..', '..', 'package.json')).then(
                    ({ version }) => version
                );
                console.log(
                    `Starting ${chalk.cyanBright('@markusjx/java')} ${chalk.greenBright(
                        'v' + version
                    )} Java definition generator`
                );
                console.log(
                    `Converting classes ${classnames
                        .map((c) => chalk.magentaBright(c))
                        .join(', ')} to typescript and saving result to ${chalk.cyanBright(path.normalize(output))}`
                );

                const spinner = ora().start();

                const resolvedImports: string[] = [];
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
                        )} Converting class ${chalk.magentaBright(lastClassResolved)}`
                    );
                };

                const TypescriptDefinitionGenerator = (await import('../TypescriptDefinitionGenerator')).default;

                for (const classname of classnames) {
                    const generator = new TypescriptDefinitionGenerator(
                        classname,
                        (name) => {
                            lastClassResolved = name;
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
                    `Success - Converted ${chalk.blueBright(numResolved)} classes in ${chalk.blueBright(
                        timeElapsed
                    )} seconds`
                );
            } catch (e) {
                console.error(e);
                if (destroyJVM) destroyJVM();
                process.exit(1);
            }
        }
    )
    .parse();
