import path from 'path';
import fs from 'fs';
import {findHome} from "../findJavaLibrary";

async function run(): Promise<void> {
    const res = await findHome(false);
    const include_dir = path.join(res, 'include');
    if (fs.existsSync(include_dir)) {
        const os_include_dir = path.join(include_dir, process.platform);
        if (fs.existsSync(os_include_dir)) {
            process.stdout.write(`${include_dir};${os_include_dir}`);
        } else {
            throw new Error("Could not find the os-specific include directory" +
                `Expected directory: ${os_include_dir}`);
        }
    } else {
        throw new Error("The include directory does not exist");
    }
}

run().then();