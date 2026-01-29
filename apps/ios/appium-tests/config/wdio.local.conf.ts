import type { Options } from '@wdio/types';
import { config as dotenvConfig } from 'dotenv';

dotenvConfig();

// Config for when Appium is already running externally
export const config: Options.Testrunner = {
	runner: 'local',
	autoCompileOpts: {
		autoCompile: true,
		tsNodeOpts: {
			transpileOnly: true,
			project: './tsconfig.json',
			esm: true,
			experimentalSpecifierResolution: 'node',
		},
	},

	specs: ['./tests/**/*.spec.ts'],
	exclude: [],

	maxInstances: 1,

	capabilities: [
		{
			platformName: 'iOS',
			'appium:automationName': 'XCUITest',
			'appium:deviceName': process.env.IOS_DEVICE_NAME || 'iPhone 17 Pro',
			'appium:platformVersion': process.env.IOS_VERSION || '26.2',
			'appium:app':
				process.env.IOS_APP_PATH || '../build/Build/Products/Debug-iphonesimulator/OFFLEASH.app',
			'appium:bundleId': 'com.offleash.ios',
			'appium:noReset': false,
			'appium:fullReset': true,
			'appium:wdaLaunchTimeout': 120000,
			'appium:wdaConnectionTimeout': 120000,
			'appium:newCommandTimeout': 300,
			'appium:screenshotQuality': 2,
			'appium:isHeadless': process.env.CI === 'true',
		},
	],

	logLevel: 'info',
	bail: 0,
	baseUrl: '',
	waitforTimeout: 30000,
	connectionRetryTimeout: 120000,
	connectionRetryCount: 3,

	// Connect to externally running Appium server
	hostname: 'localhost',
	port: 4723,
	path: '/',

	// No appium service - assumes Appium is already running
	services: [],

	framework: 'mocha',
	reporters: [
		'spec',
		[
			'allure',
			{
				outputDir: 'allure-results',
				disableWebdriverStepsReporting: true,
				disableWebdriverScreenshotsReporting: false,
			},
		],
	],

	mochaOpts: {
		ui: 'bdd',
		timeout: 120000,
		retries: process.env.CI === 'true' ? 2 : 0,
	},

	afterTest: async function (test, context, { error, passed }) {
		if (!passed) {
			await browser.takeScreenshot();
		}
	},

	onComplete: function () {
		console.log('Test execution completed');
	},
};
