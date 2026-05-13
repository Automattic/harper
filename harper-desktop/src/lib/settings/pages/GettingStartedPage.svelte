<script lang="ts">
import { Client, type AccessibilityPermissionStatus } from '$lib/client';
import { onMount } from 'svelte';
import { createInitialSettingsState, type SettingsState } from '../settings-data';

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

let state: SettingsState = createInitialSettingsState();
let accessibilityStatus: AccessibilityPermissionStatus | null = null;
let accessibilityError = '';
let isCheckingAccessibility = true;
let isRequestingAccessibility = false;
let hasRequestedAccessibility = false;
let accessibilityDiagnostics: string[] = [];

$: setupSteps = buildSetupSteps(
	state,
	accessibilityStatus,
	isCheckingAccessibility,
	isRequestingAccessibility,
	hasRequestedAccessibility,
);
$: setupCompletedCount = setupSteps.filter((step) => step.done).length;
$: setupAllDone = setupSteps.every((step) => step.done);

onMount(() => {
	recordAccessibilityDiagnostic('settings page mounted');
	void checkAccessibilityPermission();
});

function updateSetup(patch: Partial<SettingsState['setup']>) {
	state = { ...state, setup: { ...state.setup, ...patch } };
}

function enableTextEditForSetup() {
	state = {
		...state,
		integrations: { ...state.integrations, textedit: true },
		setup: { ...state.setup, integration: 'selected' },
	};
}

async function checkAccessibilityPermission() {
	isCheckingAccessibility = true;
	accessibilityError = '';
	recordAccessibilityDiagnostic('starting backend debug marker check');

	try {
		const marker = await Client.getAccessibilityPermissionDebugMarker();
		recordAccessibilityDiagnostic(`backend marker returned: ${marker}`);
		recordAccessibilityDiagnostic('starting permission status check');
		accessibilityStatus = await Client.getAccessibilityPermissionStatus();
		recordAccessibilityDiagnostic(`permission status returned: ${accessibilityStatus}`);
	} catch (error) {
		accessibilityError = `Unable to check Accessibility permission: ${error}`;
		recordAccessibilityDiagnostic(`permission check failed: ${error}`);
	} finally {
		isCheckingAccessibility = false;
		recordAccessibilityDiagnostic('permission check finished');
	}
}

async function requestAccessibilityPermission() {
	if (hasRequestedAccessibility && accessibilityStatus === 'NotGranted') {
		await checkAccessibilityPermission();
		return;
	}

	isRequestingAccessibility = true;
	accessibilityError = '';
	recordAccessibilityDiagnostic('starting permission request');

	try {
		accessibilityStatus = await Client.requestAccessibilityPermission();
		hasRequestedAccessibility = true;
		recordAccessibilityDiagnostic(`permission request returned: ${accessibilityStatus}`);
	} catch (error) {
		accessibilityError = `Unable to request Accessibility permission: ${error}`;
		recordAccessibilityDiagnostic(`permission request failed: ${error}`);
	} finally {
		isRequestingAccessibility = false;
		recordAccessibilityDiagnostic('permission request finished');
	}
}

function recordAccessibilityDiagnostic(message: string) {
	const timestamp = new Date().toLocaleTimeString();
	accessibilityDiagnostics = [...accessibilityDiagnostics.slice(-7), `${timestamp}: ${message}`];
}

function accessibilityDescription(status: AccessibilityPermissionStatus | null) {
	if (status === 'Granted') {
		return 'Harper can access text through the macOS Accessibility system.';
	}

	if (status === 'Unsupported') {
		return 'Accessibility setup is only available on macOS right now.';
	}

	return 'Open system settings and grant Harper access to the Accessibility system.';
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

	return 'Open System Settings';
}

function buildSetupSteps(
	currentState: SettingsState,
	currentAccessibilityStatus: AccessibilityPermissionStatus | null,
	currentIsCheckingAccessibility: boolean,
	currentIsRequestingAccessibility: boolean,
	currentHasRequestedAccessibility: boolean,
): SetupStep[] {
	const accessibilityDone = currentAccessibilityStatus === 'Granted';
	const integrationDone = currentState.setup.integration === 'selected';
	const testDriveDone = currentState.setup.testDrive === 'completed';
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
			required: true,
			done: accessibilityDone,
			locked: false,
			actionLabel: accessibilityActionLabel(
				currentAccessibilityStatus,
				currentIsCheckingAccessibility,
				currentIsRequestingAccessibility,
				currentHasRequestedAccessibility,
			),
			actionVariant: accessibilityDone ? 'default' : 'primary',
			action: requestAccessibilityPermission,
			actionDisabled: accessibilityActionDisabled,
		},
		{
			id: 'integration',
			title: 'Pick an app to test',
			desc: 'Start with TextEdit, then add more apps from Integrations when you are ready.',
			required: true,
			done: integrationDone,
			locked: !accessibilityDone,
			actionLabel: integrationDone ? 'Manage' : 'Browse apps',
			actionVariant: 'default',
			action: enableTextEditForSetup,
		},
		{
			id: 'test-drive',
			title: 'Take a test drive',
			desc: 'Open TextEdit, type "its not alot of fun", and watch Harper underline the mistakes.',
			required: false,
			done: testDriveDone,
			locked: !accessibilityDone || !integrationDone,
			actionLabel: testDriveDone ? 'Run again' : 'Launch TextEdit',
			actionVariant: testDriveDone ? 'default' : 'primary',
			action: () => updateSetup({ testDrive: 'completed' }),
		},
	];
}
</script>

<section>
        {#if setupAllDone}
          <div class="success-banner">
            <div class="big-mark green">
              <span class="settings-icon icon-check" aria-hidden="true"></span>
            </div>
            <div class="grow">
              <h2>You're all set</h2>
              <p>
                Harper is ready to check writing in the apps you choose. You can revisit any section
                from the sidebar.
              </p>
            </div>
            <button class="button" type="button" on:click={() => updateSetup({ testDrive: "not_started" })}>
              Walk through again
            </button>
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
                <div class="progress-fill" style={`width: ${(setupCompletedCount / setupSteps.length) * 100}%`}></div>
              </div>
              <span>{setupCompletedCount} of {setupSteps.length}</span>
            </div>
          </div>
        {/if}

        <div class="step-list">
          {#each setupSteps as step, index}
            <div class:done={step.done} class:locked={step.locked} class="step-row">
              <div class="step-dot">
                {#if step.done}
                  <span class="settings-icon icon-check" aria-hidden="true"></span>
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
                      <strong>Waiting for macOS</strong>
                      <p>After granting access in System Settings, return here and recheck permission.</p>
                    </div>
                  </div>
                {/if}

                {#if step.id === "accessibility"}
                  <div class="detected-app">
                    <div class="app-tile" style="--app-tint: #2a6bd8">D</div>
                    <div class="grow">
                      <strong>Permission diagnostics</strong>
                      {#if accessibilityDiagnostics.length === 0}
                        <p>No diagnostics recorded yet.</p>
                      {:else}
                        {#each accessibilityDiagnostics as diagnostic}
                          <p><code>{diagnostic}</code></p>
                        {/each}
                      {/if}
                    </div>
                  </div>
                {/if}

                {#if step.id === "integration" && accessibilityStatus === "Granted" && state.setup.integration !== "selected"}
                  <div class="detected-app">
                    <div class="app-tile" style="--app-tint: #5a5f68">T</div>
                    <div class="grow">
                      <strong>TextEdit detected</strong>
                      <p>A good starter app for trying Harper.</p>
                    </div>
                    <button class="button primary" type="button" on:click={enableTextEditForSetup}>
                      Enable
                    </button>
                  </div>
                {/if}
              </div>
              <button
                class={`button ${step.actionVariant === "primary" ? "primary" : ""}`}
                type="button"
                disabled={step.locked || step.actionDisabled}
                on:click={step.action}
              >
                {step.actionLabel}
              </button>
            </div>
          {/each}
        </div>

        <div class="note-strip">
          <strong>On-device by default.</strong>
          <span>Your writing stays on this Mac in this demo surface.</span>
        </div>
      </section>
