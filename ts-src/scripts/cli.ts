import yargs from 'yargs';
import TypescriptDefinitionGenerator from '../TypescriptDefinitionGenerator';
import {performance} from "perf_hooks";

interface Args {
    classnames: string[];
    output: string;
}

const importOra = (): Promise<typeof import('ora').default> => eval("import('ora').then(ora => ora.default)");

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

                const ora = await importOra();
                const spinner = ora().start();

                const resolvedImports: string[] = [];
                let numResolved: number = 0;

                for (const classname of classnames) {
                    const generator = new TypescriptDefinitionGenerator(
                        classname,
                        (name) => {
                            spinner.text = `Converting class ${name}`;
                        },
                        resolvedImports
                    );
                    const generated = await generator.generate();
                    numResolved += generated.length;
                    await TypescriptDefinitionGenerator.save(generated, output);
                }

                const timeElapsed = ((performance.now() - startTime) / 1000).toFixed(1);
                spinner.succeed(`Converted ${numResolved} classes (${timeElapsed}s)`);
            } catch (e) {
                console.error(e);
                if (destroyJVM) destroyJVM();
                process.exit(1);
            }
        }
    )
    .parse();
