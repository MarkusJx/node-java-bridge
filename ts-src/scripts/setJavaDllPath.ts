import path from 'path';
import fs from 'fs';
import { findJavaLibrary } from '../findJavaLibrary';

const outFile = path.join(__dirname, '..', '..', 'jvmLibPath.json');

async function run(): Promise<void> {
    const path = await findJavaLibrary(true);
    fs.writeFileSync(outFile, JSON.stringify(path), { flag: 'w', encoding: 'utf-8' });
}

run()
    .then()
    .catch((e) => {
        console.error(e);
        process.exit(1);
    });
