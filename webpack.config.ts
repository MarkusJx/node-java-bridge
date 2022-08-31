import path from 'path';
import { BannerPlugin } from 'webpack';
import nodeExternals from 'webpack-node-externals';

const config = (
    entry: string,
    mode: string,
    outName: string,
    banner?: string
) => ({
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
            {
                test: /native\.js/,
                loader: 'string-replace-loader',
                options: {
                    search: /require\(('java-bridge-[a-z\-0-9]+')\)/gi,
                    replace: '__non_webpack_require__($1)',
                },
            },
        ],
    },
    resolve: {
        extensions: ['.ts', '.js'],
    },
    devtool: 'source-map',
    plugins: banner
        ? [
              new BannerPlugin({
                  banner,
                  raw: true,
              }),
          ]
        : [],
});

module.exports = [
    config('./ts-src/index.ts', 'production', 'index.prod.min.js'),
    config('./ts-src/index.ts', 'development', 'index.dev.min.js'),
    config(
        './ts-src/scripts/cli.ts',
        'production',
        'java-ts-gen.js',
        '#!/usr/bin/env node'
    ),
];
