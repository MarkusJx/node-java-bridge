const path = require('path');
const nodeExternals = require('webpack-node-externals');

/**
 * @param {string} entry
 * @param {string} mode
 * @param {string} outName
 * @return {Object}
 */
const config = (entry, mode, outName) => ({
    entry,
    target: 'node',
    mode,
    externalsPresets: {
        node: true,
    },
    externals: [nodeExternals()],
    output: {
        path: path.join(__dirname, 'dist'),
        filename: outName,
        library: {
            name: 'java',
            type: 'umd',
        },
    },
    node: {
        __dirname: false,
    },
    module: {
        rules: [
            {
                test: /\.ts$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            },
            {
                test: /\.node$/,
                loader: 'node-loader',
                options: {
                    name: '[path][name].[ext]',
                },
            },
        ],
    },
    resolve: {
        extensions: ['.ts', '.js'],
    },
    //devtool: 'source-map',
});

module.exports = [
    config('./ts-src/index.ts', 'production', 'index.prod.min.js'),
    config('./ts-src/index.ts', 'development', 'index.dev.min.js'),
    config('./ts-src/scripts/cli.ts', 'production', 'cli.min.js'),
];
