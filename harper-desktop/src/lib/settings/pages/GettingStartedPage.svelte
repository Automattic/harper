<script lang="ts">
  import { createInitialSettingsState, type SettingsState } from "../settings-data";

  type SetupStep = {
    id: "accessibility" | "integration" | "test-drive";
    title: string;
    desc: string;
    required: boolean;
    done: boolean;
    locked: boolean;
    actionLabel: string;
    actionVariant: "default" | "primary";
    action: () => void;
  };

  let state: SettingsState = createInitialSettingsState();

  $: setupSteps = buildSetupSteps();
  $: setupCompletedCount = setupSteps.filter((step) => step.done).length;
  $: setupAllDone = setupSteps.every((step) => step.done);

  function updateSetup(patch: Partial<SettingsState["setup"]>) {
    state = { ...state, setup: { ...state.setup, ...patch } };
  }

  function enableTextEditForSetup() {
    state = {
      ...state,
      integrations: { ...state.integrations, textedit: true },
      setup: { ...state.setup, integration: "selected" },
    };
  }

  function buildSetupSteps(): SetupStep[] {
    const accessibilityDone = state.setup.accessibility === "granted";
    const integrationDone = state.setup.integration === "selected";
    const testDriveDone = state.setup.testDrive === "completed";

    return [
      {
        id: "accessibility",
        title: "Grant Accessibility permission",
        desc: "Open system settings and grant Harper access to the Accessibility system.",
        required: true,
        done: accessibilityDone,
        locked: false,
        actionLabel: accessibilityDone ? "Granted" : "Open System Settings",
        actionVariant: accessibilityDone ? "default" : "primary",
        action: () => updateSetup({ accessibility: "granted" }),
      },
      {
        id: "integration",
        title: "Pick an app to test",
        desc: "Start with TextEdit, then add more apps from Integrations when you are ready.",
        required: true,
        done: integrationDone,
        locked: !accessibilityDone,
        actionLabel: integrationDone ? "Manage" : "Browse apps",
        actionVariant: "default",
        action: enableTextEditForSetup,
      },
      {
        id: "test-drive",
        title: "Take a test drive",
        desc: 'Open TextEdit, type "its not alot of fun", and watch Harper underline the mistakes.',
        required: false,
        done: testDriveDone,
        locked: !accessibilityDone || !integrationDone,
        actionLabel: testDriveDone ? "Run again" : "Launch TextEdit",
        actionVariant: testDriveDone ? "default" : "primary",
        action: () => updateSetup({ testDrive: "completed" }),
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
          {#if state.setup.accessibility !== "granted"}
            <div class="warning-banner">
              <div class="big-mark amber">!</div>
              <div>
                <strong>Harper is not checking anything yet</strong>
                <p>Grant Accessibility permission so Harper can find text and surface suggestions.</p>
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

                {#if step.id === "integration" && state.setup.accessibility === "granted" && state.setup.integration !== "selected"}
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
                disabled={step.locked}
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
