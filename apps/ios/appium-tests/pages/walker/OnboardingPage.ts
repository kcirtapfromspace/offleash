import { BasePage } from '../BasePage';

export class OnboardingPage extends BasePage {
	private createOrgButton = 'walker-create-org-button';
	private joinOrgButton = 'walker-join-org-button';
	private orgNameField = 'walker-org-name';
	private orgSubmitButton = 'walker-org-submit';
	private inviteCodeField = 'walker-org-invite-code';
	private joinSubmitButton = 'walker-org-join-submit';
	private onboardingComplete = 'walker-onboarding-complete';
	private errorBanner = 'auth-error-banner';

	async waitForPageLoad(): Promise<void> {
		await this.waitFor(this.createOrgButton);
	}

	async isDisplayed(): Promise<boolean> {
		return this.isVisible(this.createOrgButton, 5000);
	}

	async tapCreateOrganization(): Promise<void> {
		await this.tap(this.createOrgButton);
	}

	async tapJoinOrganization(): Promise<void> {
		await this.tap(this.joinOrgButton);
	}

	async enterOrganizationName(name: string): Promise<void> {
		await this.type(this.orgNameField, name);
	}

	async submitOrganization(): Promise<void> {
		await this.tap(this.orgSubmitButton);
	}

	async enterInviteCode(code: string): Promise<void> {
		await this.type(this.inviteCodeField, code);
	}

	async submitJoin(): Promise<void> {
		await this.tap(this.joinSubmitButton);
	}

	async createOrganization(name: string): Promise<void> {
		await this.tapCreateOrganization();
		await this.enterOrganizationName(name);
		await this.submitOrganization();
	}

	async joinOrganization(inviteCode: string): Promise<void> {
		await this.tapJoinOrganization();
		await this.enterInviteCode(inviteCode);
		await this.submitJoin();
	}

	async isOnboardingComplete(): Promise<boolean> {
		return this.isVisible(this.onboardingComplete, 10000);
	}

	async isErrorDisplayed(): Promise<boolean> {
		return this.isVisible(this.errorBanner, 5000);
	}
}

export const onboardingPage = new OnboardingPage();
