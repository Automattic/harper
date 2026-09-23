<script lang="ts">
import { Button, CheckIcon } from 'components';
import { onMount } from 'svelte';
import { type AccessibilityPermissionStatus, Client, type Integration } from '$lib/client';
import {
	isWindows,
	platformAccessibilityName,
	platformAppId,
	platformAppName,
	platformPrivacyText,
} from '$lib/platform';
import AppIcon from '../components/AppIcon.svelte';
import type { SectionId } from '../settings-data';

type SetupStep = {
	id: 'accessibility' | 'integration' | 'test-drive';
	title: string;
	desc: string;
	required: boolean;
	done: boolean;
	locked: boolean;
	actionLabel: string;
	actionVariant: 'default' | 'primary';
	action: () => void | Promise<void>;
	actionDisabled?: boolean;
};

export let navigateToSection: (section: SectionId) => void;

let accessibilityStatus: AccessibilityPermissionStatus | null = null;
let accessibilityError = '';
let isCheckingAccessibility = true;
let isRequestingAccessibility = false;
let hasRequestedAccessibility = false;
let integrations: Integration[] = [];
let integrationsError = '';
let isLoadingIntegrations = true;
let isEnablingStarterApp = false;
let isLaunchingStarterApp = false;
let testDriveError = '';
let isCompletingOnboarding = false;
let onboardingError = '';

$: starterIntegration = integrations.find(
	(item) =>
		item.bundle_id === platformAppId ||
		(isWindows && item.bundle_id.toLowerCase().includes('notepad')),
);
$: isStarterAppEnabled = starterIntegration?.enabled === true;

$: setupSteps = buildSetupSteps(
	accessibilityStatus,
	isCheckingAccessibility,
	isRequestingAccessibility,
	hasRequestedAccessibility,
	isStarterAppEnabled,
	isLoadingIntegrations,
	isEnablingStarterApp,
	isLaunchingStarterApp,
);
$: requiredSetupSteps = setupSteps.filter((step) => step.required);
$: setupCompletedCount = requiredSetupSteps.filter((step) => step.done).length;
$: setupAllDone =
	!isCheckingAccessibility &&
	!isLoadingIntegrations &&
	requiredSetupSteps.every((step) => step.done);

onMount(() => {
	void checkAccessibilityPermission();
	void loadIntegrations();
});

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

async function enableStarterAppForSetup() {
	isEnablingStarterApp = true;
	integrationsError = '';

	try {
		if (starterIntegration) {
			await Client.setIntegrationEnabled(starterIntegration.bundle_id, true);
			integrations = integrations.map((integration) =>
				integration.bundle_id === starterIntegration.bundle_id
					? { ...integration, enabled: true }
					: integration,
			);
		} else {
			await Client.addIntegration(platformAppId);
			integrations = [
				...integrations,
				{ bundle_id: platformAppId, enabled: true, display_name: platformAppName },
			];
		}
	} catch (error) {
		integrationsError = `Unable to enable ${platformAppName}: ${error}`;
	} finally {
		isEnablingStarterApp = false;
	}
}

async function launchStarterAppForTestDrive() {
	isLaunchingStarterApp = true;
	testDriveError = '';

	try {
		await Client.launchApp(starterIntegration?.bundle_id || platformAppId);
	} catch (error) {
		testDriveError = `Unable to launch ${platformAppName}: ${error}`;
	} finally {
		isLaunchingStarterApp = false;
	}
}

async function completeOnboarding() {
	isCompletingOnboarding = true;
	onboardingError = '';

	try {
		await Client.setOnboardingCompleted(true);
		navigateToSection('general');
	} catch (error) {
		onboardingError = `Unable to complete onboarding: ${error}`;
	} finally {
		isCompletingOnboarding = false;
	}
}

async function checkAccessibilityPermission() {
	isCheckingAccessibility = true;
	accessibilityError = '';

	try {
		accessibilityStatus = await Client.getAccessibilityPermissionStatus();

		if (accessibilityStatus === 'Granted') {
			await Client.startHighlighterService();
		}
	} catch (error) {
		accessibilityError = `Unable to check Accessibility permission: ${error}`;
	} finally {
		isCheckingAccessibility = false;
	}
}

async function requestAccessibilityPermission() {
	if (hasRequestedAccessibility && accessibilityStatus === 'NotGranted') {
		await checkAccessibilityPermission();
		return;
	}

	isRequestingAccessibility = true;
	accessibilityError = '';

	try {
		accessibilityStatus = await Client.requestAccessibilityPermission();
		hasRequestedAccessibility = true;

		if (accessibilityStatus === 'Granted') {
			await Client.startHighlighterService();
		}
	} catch (error) {
		accessibilityError = `Unable to request Accessibility permission: ${error}`;
	} finally {
		isRequestingAccessibility = false;
	}
}

function accessibilityDescription(status: AccessibilityPermissionStatus | null) {
	if (status === 'Granted') {
		return `Harper can access text through ${platformAccessibilityName}.`;
	}

	if (status === 'Unsupported') {
		return 'Accessibility setup is only available on desktop right now.';
	}

	return isWindows
		? 'Harper can access text through Windows UI Automation.'
		: 'Open system settings and grant Harper access to the Accessibility system.';
}

function accessibilityActionLabel(
	status: AccessibilityPermissionStatus | null,
	isChecking: boolean,
	isRequesting: boolean,
	hasRequested: boolean,
) {
	if (isChecking) {
		return 'Checking...';
	}

	if (isRequesting) {
		return 'Opening...';
	}

	if (status === 'Granted') {
		return 'Granted';
	}

	if (status === 'Unsupported') {
		return 'Unsupported';
	}

	if (hasRequested) {
		return 'Recheck Permission';
	}

	return isWindows ? 'Check Permission' : 'Open System Settings';
}

function buildSetupSteps(
	currentAccessibilityStatus: AccessibilityPermissionStatus | null,
	currentIsCheckingAccessibility: boolean,
	currentIsRequestingAccessibility: boolean,
	currentHasRequestedAccessibility: boolean,
	currentIsStarterAppEnabled: boolean,
	currentIsLoadingIntegrations: boolean,
	currentIsEnablingStarterApp: boolean,
	currentIsLaunchingStarterApp: boolean,
): SetupStep[] {
	const accessibilityDone = currentAccessibilityStatus === 'Granted';
	const accessibilityReady = accessibilityDone || currentAccessibilityStatus === 'Unsupported';
	const integrationDone = currentIsStarterAppEnabled;
	const accessibilityActionDisabled =
		currentIsCheckingAccessibility ||
		currentIsRequestingAccessibility ||
		currentAccessibilityStatus === 'Granted' ||
		currentAccessibilityStatus === 'Unsupported';

	return [
		{
			id: 'accessibility',
			title: 'Grant Accessibility permission',
			desc: accessibilityDescription(currentAccessibilityStatus),
			required: currentAccessibilityStatus !== 'Unsupported',
			done: accessibilityDone,
			locked: false,
			actionLabel: accessibilityActionLabel(
				currentAccessibilityStatus,
				currentIsCheckingAccessibility,
				currentIsRequestingAccessibility,
				currentHasRequestedAccessibility,
			),
			actionVariant: accessibilityReady ? 'default' : 'primary',
			action: requestAccessibilityPermission,
			actionDisabled: accessibilityActionDisabled,
		},
		{
			id: 'integration',
			title: 'Pick an app to test',
			desc: `Start with ${platformAppName}, then add more apps from Integrations when you are ready.`,
			required: true,
			done: integrationDone,
			locked: !accessibilityReady,
			actionLabel: integrationDone ? 'Manage' : 'Browse apps',
			actionVariant: 'default',
			action: () => navigateToSection('integrations'),
			actionDisabled: currentIsLoadingIntegrations || currentIsEnablingStarterApp,
		},
		{
			id: 'test-drive',
			title: 'Take a test drive',
			desc: `Open ${platformAppName}, type "its not alot of fun", and watch Harper underline the mistakes.`,
			required: false,
			done: false,
			locked: !accessibilityReady || !integrationDone,
			actionLabel: currentIsLaunchingStarterApp ? 'Launching...' : `Launch ${platformAppName}`,
			actionVariant: 'primary',
			action: launchStarterAppForTestDrive,
			actionDisabled: currentIsLaunchingStarterApp,
		},
	];
}
</script>

<section>
        {#if setupAllDone}
          <div class="success-banner">
            <div class="big-mark green">
              <CheckIcon className="control-icon" />
            </div>
            <div class="grow">
              <h2>You're all set</h2>
              <p>
                Harper is ready to check writing in the apps you choose. You can revisit any section
                from the sidebar.
              </p>
              {#if onboardingError}
                <p>{onboardingError}</p>
              {/if}
            </div>
            <Button unstyled class="button" type="button" disabled={isCompletingOnboarding} on:click={completeOnboarding}>
              {isCompletingOnboarding ? "Continuing..." : "Continue"}
            </Button>
          </div>
        {:else}
          {#if accessibilityStatus !== "Granted"}
            <div class="warning-banner">
              <div class="big-mark amber">!</div>
              <div>
                {#if isCheckingAccessibility}
                  <strong>Checking Accessibility permission</strong>
                  <p>Harper needs macOS Accessibility access before it can check other apps.</p>
                {:else if accessibilityStatus === "Unsupported"}
                  <strong>Accessibility setup is unavailable</strong>
                  <p>Harper Desktop app checking is currently only wired for macOS.</p>
                {:else}
                  <strong>Harper is not checking anything yet</strong>
                  <p>Grant Accessibility permission so Harper can find text and surface suggestions.</p>
                {/if}
              </div>
            </div>
          {/if}

          <div class="hero-copy">
            <div class="eyebrow">Getting started</div>
            <h1>Let's get Harper up and running.</h1>
            <div class="progress-row">
              <div class="progress-track">
                <div class="progress-fill" style={`width: ${(setupCompletedCount / requiredSetupSteps.length) * 100}%`}></div>
              </div>
              <span>{setupCompletedCount} of {requiredSetupSteps.length}</span>
            </div>
          </div>
        {/if}

        <div class="step-list">
          {#each setupSteps as step, index}
            <div class:done={step.done} class:locked={step.locked} class="step-row">
              <div class="step-dot">
                {#if step.done}
                  <CheckIcon className="control-icon" />
                {:else}
                  {index + 1}
                {/if}
              </div>
              <div class="grow">
                <div class="step-heading">
                  <strong>{step.title}</strong>
                  {#if !step.required && !step.done}
                    <span class="pill">Optional</span>
                  {/if}
                </div>
                <p>{step.desc}</p>

                {#if step.id === "accessibility" && accessibilityError}
                  <div class="detected-app">
                    <div class="big-mark amber">!</div>
                    <div class="grow">
                      <strong>Permission check failed</strong>
                      <p>{accessibilityError}</p>
                    </div>
                  </div>
                {:else if step.id === "accessibility" && hasRequestedAccessibility && accessibilityStatus === "NotGranted"}
                  <div class="detected-app">
                    <div class="app-tile" style="--app-tint: #b06a1b">A</div>
                    <div class="grow">
                      <strong>Waiting for {isWindows ? 'Windows' : 'macOS'}</strong>
                      <p>After granting access in settings, return here and recheck permission.</p>
                    </div>
                  </div>
                {/if}

                {#if step.id === "test-drive" && testDriveError}
                  <div class="detected-app">
                    <div class="big-mark amber">!</div>
                    <div class="grow">
                      <strong>{platformAppName} launch failed</strong>
                      <p>{testDriveError}</p>
                    </div>
                  </div>
                {/if}

                {#if step.id === "integration" && integrationsError}
                  <div class="detected-app">
                    <div class="big-mark amber">!</div>
                    <div class="grow">
                      <strong>Integration update failed</strong>
                      <p>{integrationsError}</p>
                    </div>
                  </div>
                {:else if step.id === "integration" && accessibilityStatus === "Granted" && isLoadingIntegrations}
                  <div class="detected-app">
                    <AppIcon bundleId={platformAppId} name={platformAppName} />
                    <div class="grow">
                      <strong>Checking {platformAppName}</strong>
                      <p>Loading integration state...</p>
                    </div>
                  </div>
                {:else if step.id === "integration" && accessibilityStatus === "Granted" && isStarterAppEnabled}
                  <div class="detected-app">
                    <AppIcon bundleId={platformAppId} name={platformAppName} />
                    <div class="grow">
                      <strong>{platformAppName} enabled</strong>
                      <p>Harper is configured to check {platformAppName}.</p>
                    </div>
                  </div>
                {:else if step.id === "integration" && accessibilityStatus === "Granted"}
                  <div class="detected-app">
                    <AppIcon bundleId={platformAppId} name={platformAppName} />
                    <div class="grow">
                      <strong>{platformAppName} detected</strong>
                      <p>A good starter app for trying Harper.</p>
                    </div>
                    <Button unstyled class="button primary" type="button" disabled={isEnablingStarterApp} on:click={enableStarterAppForSetup}>
                      {isEnablingStarterApp ? "Enabling..." : "Enable"}
                    </Button>
                  </div>
                {/if}
              </div>
              <Button
                unstyled
                class={`button ${step.actionVariant === "primary" ? "primary" : ""}`}
                type="button"
                disabled={step.locked || step.actionDisabled}
                on:click={step.action}
              >
                {step.actionLabel}
              </Button>
            </div>
          {/each}
        </div>

        <div class="note-strip">
          <strong>On-device by default.</strong>
          <span>Your writing stays on {platformPrivacyText} in this demo surface.</span>
        </div>
      </section>
