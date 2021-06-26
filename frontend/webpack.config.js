const MiniCssExtractPlugin = require('mini-css-extract-plugin');
const sveltePreprocess = require( "svelte-preprocess")// add this
const { merge } = require('webpack-merge');
const svelteConfig = require("./svelte.config");
const path = require('path');
const webpack = require("webpack");

const mode = process.env.NODE_ENV || 'development';
const prod = mode === 'production';

const wp_dev = {
	entry: {
		'build/bundle': ['./src/index.ts']
	},
	resolve: {
		alias: {
			svelte: path.dirname(require.resolve('svelte/package.json'))
		},
		extensions: ['.mjs', '.js', '.svelte','.ts'],
		mainFields: ['svelte', 'browser', 'module', 'main'],
        fallback:{ "assert": require.resolve("assert/") },
	},
	output: {
		path: path.join(__dirname, '/public'),
		filename: '[name].js',
		chunkFilename: '[name].[id].js'
	},
	module: {
		rules: [
			{
				test: /\.svelte$/,
				use: {
					loader: 'svelte-loader',
					options: {
						compilerOptions: {
							dev: !prod
						},
						emitCss: prod,
						hotReload: !prod,
						preprocess: svelteConfig.preprocess
					}
				}
			},
			{
				test: /\.css$/,
				use: [
					MiniCssExtractPlugin.loader,
					'css-loader'
				]
			},
			{
				// required to prevent errors from Svelte on Webpack 5+
				test: /node_modules\/svelte\/.*\.mjs$/,
				resolve: {
					fullySpecified: false
				}
			}, 
            {
				test: /\.ts$/,
				use: 'ts-loader',
				exclude: /node_modules/
            },
		]
	},
	mode,
	plugins: [
		new MiniCssExtractPlugin({
			filename: '[name].css'
		}),
        new webpack.ProvidePlugin({
            process: 'process/browser',
        }),
	],
	devtool: prod ? false : 'source-map',
	devServer: {
		hot: true
	}
}

module.exports = wp_dev
