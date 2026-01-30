export const environment = {
	apiBaseUrl: process.env.API_BASE_URL || 'https://api-staging.offleash.com',
	testEnv: process.env.TEST_ENV || 'local',
	allureResultsDir: process.env.ALLURE_RESULTS_DIR || './allure-results',
	screenshotOnFailure: process.env.SCREENSHOT_ON_FAILURE !== 'false',
	videoRecording: process.env.VIDEO_RECORDING === 'true',
};
