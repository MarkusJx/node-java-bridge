import isCi from 'is-ci';

export const shouldIncreaseTimeout =
    isCi && (process.arch === 'arm64' || process.arch === 'arm');

console.log('Process arch:', process.arch);
console.log('Process platform:', process.platform);
