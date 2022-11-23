import { TypescriptBulkDefinitionGenerator } from '../.';
import path from 'path';
import fs from 'fs';

const gen = new TypescriptBulkDefinitionGenerator();
const outDir = path.join(__dirname, 'javaDefinitions');

async function run() {
    if (fs.existsSync(outDir)) {
        return;
    }

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
        (classname) => console.log(`Converting '${classname}' to typescript`)
    );

    console.log('Saving results...');
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
}

run().catch((err) => {
    console.error(err);
    process.exit(1);
});
