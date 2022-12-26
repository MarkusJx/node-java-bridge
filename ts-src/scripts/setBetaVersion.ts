import { execSync, spawn } from 'child_process';
import semver from 'semver';
import { name, version as packageVersion } from '../../package.json';
import path from 'path';
import which from 'which';

const npm = which.sync('npm');
const npmVersion = semver.parse(
    execSync(`${npm} view ${name} version`, {
        encoding: 'utf8',
    }).trim()
);
const version = semver.parse(packageVersion);

if (!npmVersion || !version) {
    throw new Error('Could not get current version');
}

console.log('Current version in package.json:', version.format());
console.log('Current version on npm:', npmVersion.format());

const getNext = (version: semver.SemVer): semver.SemVer => {
    console.log('Incrementing version:', version.format());

    let next: semver.SemVer | null;
    if (version.prerelease.length === 0) {
        next = semver.parse(`${version.format()}-beta.0`);
    } else {
        next = version.inc('prerelease', 'beta');
    }

    if (!next) {
        throw new Error('Could not increment version');
    }

    return next;
};

let nextVersion: semver.SemVer | null;
if (npmVersion.compareMain(version) > 0) {
    nextVersion = getNext(npmVersion);
} else if (npmVersion.compareMain(version) < 0) {
    nextVersion = getNext(version);
} else if (
    npmVersion.prerelease.length !== 0 ||
    version.prerelease.length !== 0
) {
    if (npmVersion.prerelease.length !== 0 && version.prerelease.length === 0) {
        nextVersion = getNext(npmVersion);
    } else if (
        npmVersion.prerelease.length === 0 &&
        version.prerelease.length !== 0
    ) {
        nextVersion = getNext(version);
    } else {
        nextVersion =
            npmVersion.comparePre(version) > 0
                ? getNext(npmVersion)
                : getNext(version);
    }
} else {
    nextVersion = getNext(version);
}

if (!nextVersion) {
    throw new Error('Could not determine next version');
}

console.log(`Setting version to ${nextVersion}`);
const child = spawn(
    npm,
    [
        'version',
        nextVersion.format(),
        '--no-git-tag-version',
        '-f',
        '--allow-same-version',
    ],
    {
        cwd: path.join(__dirname, '..', '..'),
    }
);

child.stdout.on('data', (data) => {
    console.log(data.toString());
});

child.stderr.on('data', (data) => {
    console.error(data.toString());
});

child.on('close', (code) => {
    if (code !== 0) {
        console.error(`npm version exited with code ${code}`);
        process.exit(1);
    }
});
