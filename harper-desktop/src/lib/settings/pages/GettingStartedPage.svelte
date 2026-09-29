<script lang="ts">
import { isTauri } from '@tauri-apps/api/core';
import { platform } from '@tauri-apps/plugin-os';
import { Button, SlideDeck } from 'components';
import { onMount } from 'svelte';
import { type AccessibilityPermissionStatus, Client, type Integration } from '$lib/client';
import AppIcon from '../components/AppIcon.svelte';

/** Slide identities determine setup gates; only the welcome and test drive need no action. */
type OnboardingSlide = {
	id: 'welcome' | 'accessibility' | 'integration' | 'test-drive' | 'ready';
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
		title: 'Accessibility',
		lede: 'To be able to read and write text to your favorite text editors, you need to grant Harper the permission to do so.\n\nNone of your text will leave your device.',
	},
	{
		id: 'integration',
		title: 'Enable TextEdit',
		lede: 'The Harper community is constantly adding support for new text editors. If we haven\'t marked a text editor as "supported" already, you can override this option yourself.\n\nLet\'s try that now. Go ahead and enable "TextEdit". Once you do, Harper will start checking your grammar in that app.',
	},
	{
		id: 'test-drive',
		title: 'Try Harper',
		lede: 'Now that you\'ve enabled TextEdit, go ahead and open it and write something like, "This is an test."\n\nYou should see Harper jump in to fix that mistake.',
	},
	{
		id: 'ready',
		title: 'Ready',
		lede: "Okay! Now we're ready to go.\n\nDon't let your dreams be dreams. Write anything, anywhere and Harper will be there to catch your mistakes.",
	},
];

export let onComplete: () => void;

let step = 0;
let isMacOS = false;
let accessibilityStatus: AccessibilityPermissionStatus | null = null;
let setupError = '';
let isPreparingService = true;
let hasRequestedAccessibility = false;
let integrations: Integration[] = [];
let integrationsError = '';
let isLoadingIntegrations = true;
let isEnablingTextEdit = false;
let isLaunchingTextEdit = false;
let testDriveError = '';
let isCompletingOnboarding = false;
let onboardingError = '';

$: slides = allSlides.filter((slide) => slide.id !== 'accessibility' || isMacOS);
$: textEditIntegration = integrations.find((item) => item.bundle_id === 'com.apple.TextEdit');
$: isTextEditEnabled = textEditIntegration?.enabled === true;
$: accessibilityReady =
	(!isMacOS || accessibilityStatus === 'Granted') && !isPreparingService && !setupError;
$: integrationReady =
	isTextEditEnabled && !isLoadingIntegrations && !isEnablingTextEdit && !integrationsError;
$: nextDisabled = !canAdvance(slides[step].id, accessibilityReady, integrationReady, isMacOS);

onMount(() => {
	// Use the native platform in Tauri; user-agent detection is only for browser previews.
	isMacOS = isTauri() ? platform() === 'macos' : navigator.userAgent.includes('Macintosh');
	void prepareService();
	void loadIntegrations();
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

/** Persist completion only after mandatory setup; leave the deck only when the save succeeds. */
async function completeOnboarding() {
	if (!accessibilityReady || !integrationReady || isCompletingOnboarding) return;
	isCompletingOnboarding = true;
	onboardingError = '';

	try {
		await Client.setOnboardingCompleted(true);
		onComplete();
	} catch (error) {
		onboardingError = `Unable to complete onboarding: ${error}`;
	} finally {
		isCompletingOnboarding = false;
	}
}

/** Start Harper on every platform, checking or requesting Accessibility permission only on macOS. */
async function prepareService(request = false) {
	isPreparingService = true;
	setupError = '';
	accessibilityStatus = null;

	try {
		if (isMacOS) {
			if (request) {
				accessibilityStatus = await Client.requestAccessibilityPermission();
				hasRequestedAccessibility = true;
			} else {
				accessibilityStatus = await Client.getAccessibilityPermissionStatus();
			}
		}

		if (
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
    onBack={() => {
      if (!isCompletingOnboarding) step--;
    }}
    onNext={() => step++}
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
        <p role="status">
          {#if isPreparingService}
            Checking Accessibility access...
          {:else if accessibilityReady}
            Accessibility access granted. Harper is ready to check your writing.
          {:else if accessibilityStatus === 'Unsupported'}
            Accessibility setup is unavailable on this platform. Setup cannot continue here.
          {:else if hasRequestedAccessibility && accessibilityStatus === 'NotGranted'}
            Enable Harper in System Settings → Privacy &amp; Security → Accessibility, then recheck permission.
          {/if}
        </p>
        {#if setupError}
          <p role="alert">{setupError}</p>
        {/if}
        <Button
          disabled={isPreparingService || accessibilityReady || accessibilityStatus === 'Unsupported'}
          on:click={() => prepareService(!hasRequestedAccessibility && accessibilityStatus !== 'Granted')}
        >
          {#if isPreparingService}
            Checking...
          {:else if accessibilityReady}
            Granted
          {:else if accessibilityStatus === 'Unsupported'}
            Unavailable
          {:else if hasRequestedAccessibility || accessibilityStatus === 'Granted'}
            Recheck Permission
          {:else}
            Open System Settings
          {/if}
        </Button>
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
          <Button color="light" disabled={isLoadingIntegrations || isEnablingTextEdit} on:click={loadIntegrations}>
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
        {#if onboardingError}
          <p role="alert">{onboardingError}</p>
        {/if}
        <Button disabled={isCompletingOnboarding || !accessibilityReady || !integrationReady} on:click={completeOnboarding}>
          {isCompletingOnboarding ? 'Finishing...' : 'Finish'}
        </Button>
      </div>
    {/if}
  </SlideDeck>
</div>
