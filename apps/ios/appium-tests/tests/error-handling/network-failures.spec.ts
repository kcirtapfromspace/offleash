import { ServicesPage } from '../../pages/customer/ServicesPage.js';
import {
	expectVisible,
	loginAsCustomer,
	resetApp,
	setNetworkOffline,
	setNetworkOnline,
} from '../../utils/helpers.js';
import { ids } from '../../utils/ids.js';

describe('Error Handling - Network', () => {
	beforeEach(async () => {
		await resetApp();
		await loginAsCustomer();
	});

	it('ERR-001 Network Offline', async () => {
		await setNetworkOffline();
		await ServicesPage.open();
		await expectVisible(ids.errors.offlineBanner);
		await setNetworkOnline();
	});

	it('ERR-002 API Timeout', async () => {
		await ServicesPage.open();
		await expectVisible(ids.errors.timeoutBanner);
	});

	it('ERR-003 401 Unauthorized', async () => {
		await expectVisible(ids.errors.unauthorizedBanner);
	});

	it('ERR-004 500 Server Error', async () => {
		await expectVisible(ids.errors.serverErrorBanner);
	});
});
