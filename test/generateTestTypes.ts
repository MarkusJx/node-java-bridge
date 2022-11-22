import {TypescriptBulkDefinitionGenerator} from '../.';
import path from 'path';

const gen = new TypescriptBulkDefinitionGenerator();

gen.generate([
    'java.io.FileOutputStream',
    'java.util.jar.JarOutputStream',
    'java.util.jar.Manifest',
    'java.util.jar.Attributes',
    'java.io.File',
    'javax.tools.ToolProvider',
    'java.net.URLClassLoader',
], (classname) => console.log(`Converting '${classname}' to typescript`))
    .then(async () => {
        console.log("Saving results...");
        gen.moduleDeclarations.forEach((declaration, i) => {
            gen.moduleDeclarations[i] = {
                ...declaration,
                contents: declaration.contents.replaceAll('from "java-bridge";', `from ${JSON.stringify(path.join(__dirname, '..'))};`),
            };
        });
        await gen.save(path.join(__dirname, 'javaDefinitions'));
    })
    .catch((err) => {
        console.error(err);
        process.exit(1);
    });
