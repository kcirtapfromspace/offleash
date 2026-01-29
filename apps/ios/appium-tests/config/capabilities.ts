if (!process.env.OFFLEASH_TEST_AUTH) {
	process.env.OFFLEASH_TEST_AUTH = 'mock';
}

const processArguments = process.env.OFFLEASH_TEST_AUTH
	? {
			env: { OFFLEASH_TEST_AUTH: process.env.OFFLEASH_TEST_AUTH },
			args: ['-OFFLEASH_TEST_AUTH', process.env.OFFLEASH_TEST_AUTH],
		}
	: undefined;

export const iosCapabilities = {
	platformName: 'iOS',
	'appium:automationName': 'XCUITest',
	'appium:deviceName': process.env.IOS_SIMULATOR || 'iPhone 17',
	'appium:platformVersion': process.env.IOS_VERSION || '26.2',
	'appium:app': process.env.IOS_APP_PATH || './build/OFFLEASH.app',
	'appium:bundleId': process.env.IOS_BUNDLE_ID || 'com.offleash.ios',
	'appium:noReset': false,
	'appium:fullReset': false,
	'appium:wdaLaunchTimeout': 120000,
	'appium:wdaConnectionTimeout': 120000,
	'appium:newCommandTimeout': 300,
	'appium:shouldTerminateApp': true,
	'appium:useNewWDA': true,
	'appium:waitForIdleTimeout': 10000,
	'appium:screenshotQuality': 2,
	...(processArguments ? { 'appium:processArguments': processArguments } : {}),
};
