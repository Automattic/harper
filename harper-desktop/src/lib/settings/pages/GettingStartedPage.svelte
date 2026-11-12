<script lang="ts">
import { isTauri } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { platform } from '@tauri-apps/plugin-os';
import { Button, SlideDeck } from 'components';
import { onMount } from 'svelte';
import {
	type AccessibilityPermissionStatus,
	Client,
	type Integration,
	type OnboardingState,
} from '$lib/client';
import AppIcon from '../components/AppIcon.svelte';

/** Slide identities determine setup gates; only the welcome and test drive need no action. */
type OnboardingSlide = {
	id: Exclude<OnboardingState, 'complete'>;
	title: string;
	lede: string;
};

const allSlides: OnboardingSlide[] = [
	{
		id: 'welcome',
		title: 'Welcome',
		lede: "Before you can start writing with Harper, we need to do a little housekeeping.\n\nDon't worry, this should only take a minute.",
	},
	{
		id: 'accessibility',
		title: 'Give Harper Permission',
		lede: "Harper needs permission to read and write text in your favorite text editors. None of your text will leave your device.\n\nFind Harper in System Settings → Privacy & Security → Device Control and Data Access. Grant the permission in system settings, then come back here. If you've already granted permission and it isn't being reflected here, you may need to restart Harper.",
	},
	{
		id: 'integration',
		title: 'Enable TextEdit',
		lede: 'The Harper community is constantly adding support for new text editors. If we haven\'t marked a text editor as "supported" already, you can override this option yourself.\n\nLet\'s try that now. Go ahead and enable "TextEdit". Once you do, Harper will start checking your grammar in that app.',
	},
	{
		id: 'test-drive',
		title: 'Try Harper',
		lede: 'Now that you\'ve enabled TextEdit, go ahead and open it. Write something like, "This is an test."\n\nYou should see Harper jump in to fix that mistake.',
	},
	{
		id: 'ready',
		title: 'Ready',
		lede: "Okay! Now we're ready to go.\n\nDon't let your dreams be dreams. Write anything, anywhere and Harper will be there to catch your mistakes.",
	},
];

export let initialState: Exclude<OnboardingState, 'complete'>;
export let onComplete: () => void;

let step = 0;
let isMacOS = false;
let accessibilityStatus: AccessibilityPermissionStatus | null = null;
let setupError = '';
let isPreparingService = false;
let integrations: Integration[] = [];
let integrationsError = '';
let isLoadingIntegrations = true;
let isEnablingTextEdit = false;
let isLaunchingTextEdit = false;
let testDriveError = '';
let isCompletingOnboarding = false;
let onboardingError = '';
let isRestarting = false;
let restartError = '';
let isSavingStep = false;

$: slides = allSlides.filter((slide) => slide.id !== 'accessibility' || isMacOS);
$: textEditIntegration = integrations.find((item) => item.bundle_id === 'com.apple.TextEdit');
$: isTextEditEnabled = textEditIntegration?.enabled === true;
$: accessibilityReady =
	(!isMacOS || accessibilityStatus === 'Granted') && !isPreparingService && !setupError;
$: integrationReady =
	isTextEditEnabled && !isLoadingIntegrations && !isEnablingTextEdit && !integrationsError;
$: nextDisabled =
	isRestarting ||
	isSavingStep ||
	isCompletingOnboarding ||
	!canAdvance(slides[step].id, accessibilityReady, integrationReady, isMacOS);

onMount(() => {
	// Use the native platform in Tauri; user-agent detection is only for browser previews.
	isMacOS = isTauri() ? platform() === 'macos' : navigator.userAgent.includes('Macintosh');
	const availableSlides = allSlides.filter((slide) => slide.id !== 'accessibility' || isMacOS);
	// Accessibility is skipped when a config is resumed on a non-macOS platform.
	const restoredState = initialState === 'accessibility' && !isMacOS ? 'integration' : initialState;
	step = availableSlides.findIndex((slide) => slide.id === restoredState);
	void prepareService();
	void loadIntegrations();
	const permissionTimer = isMacOS ? setInterval(() => void prepareService(), 2000) : undefined;
	return () => clearInterval(permissionTimer);
});

/** Gate service startup on Welcome outside macOS, or on Accessibility on macOS; the trial is optional. */
function canAdvance(
	slide: OnboardingSlide['id'],
	accessReady: boolean,
	appReady: boolean,
	needsPermission: boolean,
) {
	if (slide === 'welcome') return needsPermission || accessReady;
	if (slide === 'accessibility') return accessReady;
	return accessReady && appReady;
}

async function loadIntegrations() {
	isLoadingIntegrations = true;
	integrationsError = '';

	try {
		integrations = await Client.getIntegrations();
	} catch (error) {
		integrationsError = `Unable to load integrations: ${error}`;
	} finally {
		isLoadingIntegrations = false;
	}
}

async function enableTextEditForSetup() {
	if (!accessibilityReady || isLoadingIntegrations || isEnablingTextEdit) return;
	isEnablingTextEdit = true;
	integrationsError = '';

	try {
		if (textEditIntegration) {
			await Client.setIntegrationEnabled('com.apple.TextEdit', true);
		} else {
			await Client.addIntegration('com.apple.TextEdit');
		}
		await loadIntegrations();
	} catch (error) {
		integrationsError = `Unable to enable TextEdit: ${error}`;
	} finally {
		isEnablingTextEdit = false;
	}
}

async function launchTextEditForTestDrive() {
	if (!accessibilityReady || !integrationReady || isLaunchingTextEdit) return;
	isLaunchingTextEdit = true;
	testDriveError = '';

	try {
		await Client.launchApp('com.apple.TextEdit');
	} catch (error) {
		testDriveError = `Unable to launch TextEdit: ${error}`;
	} finally {
		isLaunchingTextEdit = false;
	}
}

/** Persist navigation before showing the next page so a relaunch restores it. */
async function changeStep(direction: -1 | 1) {
	if (isSavingStep || isCompletingOnboarding || isRestarting) return;
	if (direction === 1 && nextDisabled) return;
	const nextStep = step + direction;
	const nextSlide = slides[nextStep];
	if (!nextSlide) return;
	isSavingStep = true;
	onboardingError = '';
	try {
		await Client.setOnboardingState(nextSlide.id);
		step = nextStep;
	} catch (error) {
		onboardingError = `Unable to save onboarding progress: ${error}. Please try again.`;
	} finally {
		isSavingStep = false;
	}
}

/** Leave the deck only after completion has been saved successfully. */
async function completeOnboarding() {
	if (
		!accessibilityReady ||
		!integrationReady ||
		isCompletingOnboarding ||
		isSavingStep ||
		isRestarting
	)
		return;
	isCompletingOnboarding = true;
	onboardingError = '';

	try {
		await Client.setOnboardingState('complete');
		onComplete();
	} catch (error) {
		onboardingError = `Unable to complete onboarding: ${error}`;
	} finally {
		isCompletingOnboarding = false;
	}
}

/** Save the current page before restarting the parent; never restart from a permission check. */
async function restartHarper() {
	if (isRestarting || isSavingStep || isCompletingOnboarding) return;
	isRestarting = true;
	restartError = '';
	try {
		await Client.setOnboardingState(slides[step].id);
		await Client.restartHarper();
	} catch (error) {
		restartError = `Unable to restart Harper: ${error}. If needed, quit Harper and open the installed application again.`;
		isRestarting = false;
	}
}

async function openAccessibilitySettings() {
	try {
		await openUrl(
			'x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_Accessibility',
		);
	} catch (error) {
		setupError = `Unable to open System Settings: ${error}`;
	}
}

/** Check permission without overlapping requests; start Harper when access becomes available. */
async function prepareService() {
	if (isPreparingService || isRestarting) return;
	const shouldStartService = accessibilityStatus !== 'Granted' || !!setupError;
	isPreparingService = true;
	setupError = '';

	try {
		if (isMacOS) {
			accessibilityStatus = await Client.getAccessibilityPermissionStatus();
		}

		if (
			shouldStartService &&
			(!isMacOS || accessibilityStatus === 'Granted') &&
			!(await Client.startHighlighterService())
		) {
			throw new Error('The Harper service did not start. Please try again.');
		}
	} catch (error) {
		setupError = `Unable to ${isMacOS ? 'set up Accessibility' : 'start Harper'}: ${error}`;
	} finally {
		isPreparingService = false;
	}
}
</script>

<div class="onboarding-shell">
  <SlideDeck
    title={slides[step].title}
    lede={slides[step].lede}
    slideProgress={step / (slides.length - 1)}
    {nextDisabled}
    onBack={() => void changeStep(-1)}
    onNext={() => void changeStep(1)}
  >
    {#if slides[step].id === 'welcome' && !isMacOS}
      {#if isPreparingService}
        <div class="onboarding-actions">
          <p role="status">Starting Harper...</p>
        </div>
      {:else if setupError}
        <div class="onboarding-actions">
          <p role="alert">{setupError}</p>
          <Button on:click={() => prepareService()}>Retry Starting Harper</Button>
        </div>
      {/if}
    {:else if slides[step].id === 'accessibility'}
      <div class="onboarding-actions">
        <div class="permission-actions">
          <Button on:click={openAccessibilitySettings}>Open System Settings</Button>
          <span role="status">
            {#if accessibilityStatus === 'Granted'}
              Permission granted
            {:else if accessibilityStatus === 'NotGranted'}
              Permission not granted
            {:else if accessibilityStatus === 'Unsupported'}
              Permission check unavailable
            {:else}
              Checking permission...
            {/if}
          </span>
          {#if accessibilityStatus === 'NotGranted'}
            <button type="button" class="restart-link" disabled={isRestarting} on:click={restartHarper}>
              {isRestarting ? 'Restarting Harper...' : 'Restart Harper'}
            </button>
          {/if}
        </div>
        {#if restartError}
          <p role="alert">{restartError}</p>
        {/if}
        {#if setupError}
          <p role="alert">{setupError}</p>
        {/if}
      </div>
    {:else if slides[step].id === 'integration'}
      <div class="onboarding-actions">
        <p role="status">
          {#if isLoadingIntegrations}
            Loading integration state...
          {:else if isEnablingTextEdit}
            Enabling TextEdit...
          {:else if integrationReady}
            TextEdit enabled. Harper will check your writing in this app.
          {/if}
        </p>
        {#if integrationsError}
          <p role="alert">{integrationsError}</p>
          <Button color="white" disabled={isLoadingIntegrations || isEnablingTextEdit} on:click={loadIntegrations}>
            Retry Loading Integrations
          </Button>
        {/if}
        <div class="onboarding-app">
          <Button
            disabled={!accessibilityReady || isLoadingIntegrations || isEnablingTextEdit || isTextEditEnabled || !!integrationsError}
            on:click={enableTextEditForSetup}
          >
            {isEnablingTextEdit ? 'Enabling...' : isTextEditEnabled ? 'Enabled' : 'Enable TextEdit'}
          </Button>
          <AppIcon bundleId="com.apple.TextEdit" name="TextEdit" />
          <strong>TextEdit</strong>
        </div>
      </div>
    {:else if slides[step].id === 'test-drive'}
      <div class="onboarding-actions">
        {#if testDriveError}
          <p role="alert">{testDriveError}</p>
        {/if}
        <Button disabled={isLaunchingTextEdit} on:click={launchTextEditForTestDrive}>
          {isLaunchingTextEdit ? 'Launching...' : 'Launch TextEdit'}
        </Button>
      </div>
    {:else if slides[step].id === 'ready'}
      <div class="onboarding-actions">
        <Button disabled={isCompletingOnboarding || !accessibilityReady || !integrationReady} on:click={completeOnboarding}>
          {isCompletingOnboarding ? 'Finishing...' : 'Finish'}
        </Button>
      </div>
    {/if}
    {#if onboardingError}
      <p role="alert">{onboardingError}</p>
    {/if}
  </SlideDeck>
</div>

<style>
.permission-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 1rem;
}

.restart-link {
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--settings-accent);
  text-decoration: underline;
  cursor: pointer;
}

.restart-link:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
