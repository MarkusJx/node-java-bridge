import isCi from 'is-ci';

export const shouldIncreaseTimeout =
    isCi && (process.arch === 'arm64' || process.arch === 'arm');
