<script lang="ts">
  const shortcutItems = [
    { id: "show-menu", label: "Show Harper menu", keys: ["Shift", "Cmd", "H"] },
    { id: "quick-review", label: "Open quick review", keys: ["Ctrl", "Cmd", "Space"] },
    { id: "accept-last", label: "Apply last suggestion", keys: ["Ctrl", "E"] },
    { id: "dismiss-last", label: "Dismiss last suggestion", keys: ["Ctrl", "D"] },
    { id: "toggle-pause", label: "Pause or resume checking", keys: ["Ctrl", "Shift", "P"] },
    { id: "add-word", label: "Add word to dictionary", keys: ["Ctrl", "Shift", "A"] },
    { id: "next-suggestion", label: "Jump to next suggestion", keys: [] },
  ];

  let globalShortcuts = true;
  let activationKey = "off";
</script>

<section>
        <div class="stanza">
          <div class="eyebrow">Keyboard Shortcuts</div>
          <p class="section-copy">
            Global shortcuts work from anywhere on macOS. Click a shortcut to preview recording.
          </p>

          <div class="shortcut-list">
            {#each shortcutItems as item}
              <button class="shortcut-row" type="button" disabled title="Not wired yet">
                <span>{item.label}</span>
                {#if item.keys.length > 0}
                  <span class="kbd-group">
                    {#each item.keys as key}
                      <kbd>{key}</kbd>
                    {/each}
                  </span>
                {:else}
                  <em>Click to record...</em>
                {/if}
              </button>
            {/each}
          </div>

          <div class="row top">
            <div>
              <strong>Allow shortcuts while other apps are focused</strong>
              <p>When off, Harper shortcuts only work while the Harper window is active.</p>
            </div>
            <button
              class:checked={globalShortcuts}
              class="checkbox"
              type="button"
              role="checkbox"
              disabled
              title="Not wired yet"
              aria-checked={globalShortcuts}
            >
              {#if globalShortcuts}<span class="settings-icon icon-check" aria-hidden="true"></span>{/if}
            </button>
          </div>
        </div>

        <div class="divider"></div>

        <div class="stanza">
          <div class="eyebrow">Activation</div>
          <div class="row top">
            <div>
              <strong>Activation key</strong>
              <p>Require a modifier key to enable Harper checking in a window.</p>
            </div>
            <select
              class="select"
              disabled
              title="Not wired yet"
              bind:value={activationKey}
            >
              <option value="off">Off</option>
              <option value="option">Option</option>
              <option value="control">Control</option>
              <option value="shift">Shift</option>
            </select>
          </div>
        </div>
      </section>
