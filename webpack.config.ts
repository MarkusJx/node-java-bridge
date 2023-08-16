import path from 'path';
import nodeExternals from 'webpack-node-externals';

module.exports = {
    entry: './ts-src/index.ts',
    target: 'node',
    mode: 'production',
    externalsPresets: {
        node: true,
    },
    externals: [nodeExternals()],
    output: {
        path: path.join(__dirname, 'dist'),
        filename: 'index.prod.min.js',
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
};
