import { destroyJVM } from '../.';
import yargs from 'yargs';
import TypescriptDefinitionGenerator from '../TypescriptDefinitionGenerator';

interface Args {
    classname: string;
}

yargs
    .command<Args>(
        '* <classname>',
        false,
        (command) => {
            command.positional('classname', {
                describe: 'The fully qualified class name',
                type: 'string',
            });
        },
        async ({ classname }) => {
            try {
                const generated = new TypescriptDefinitionGenerator(classname).generate();
                await TypescriptDefinitionGenerator.save(generated, 'C:\\Users\\marku\\Desktop\\test');
            } catch (e) {
                console.error(e);
                destroyJVM();
                process.exit(1);
            }
        }
    )
    .parse();
