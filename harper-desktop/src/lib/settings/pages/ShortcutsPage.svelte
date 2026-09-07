<script lang="ts">
import { Button, Checkbox, CheckIcon, Select, SettingRow } from 'components';

const shortcutItems = [
	{ id: 'show-menu', label: 'Show Harper menu', keys: ['Shift', 'Cmd', 'H'] },
	{ id: 'quick-review', label: 'Open quick review', keys: ['Ctrl', 'Cmd', 'Space'] },
	{ id: 'accept-last', label: 'Apply last suggestion', keys: ['Ctrl', 'E'] },
	{ id: 'dismiss-last', label: 'Dismiss last suggestion', keys: ['Ctrl', 'D'] },
	{ id: 'toggle-pause', label: 'Pause or resume checking', keys: ['Ctrl', 'Shift', 'P'] },
	{ id: 'add-word', label: 'Add word to dictionary', keys: ['Ctrl', 'Shift', 'A'] },
	{ id: 'next-suggestion', label: 'Jump to next suggestion', keys: [] },
];

let globalShortcuts = true;
let activationKey = 'off';
</script>

<section>
        <div class="stanza">
          <div class="eyebrow">Keyboard Shortcuts</div>
          <p class="section-copy">
            Global shortcuts work from anywhere on macOS. Click a shortcut to preview recording.
          </p>

          <div class="shortcut-list">
            {#each shortcutItems as item}
              <Button unstyled class="shortcut-row" type="button" disabled title="Not wired yet">
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
              </Button>
            {/each}
          </div>

          <SettingRow top>
            <strong>Allow shortcuts while other apps are focused</strong>
            <p>When off, Harper shortcuts only work while the Harper window is active.</p>
            <Checkbox
              slot="control"
              appearance="settings"
              checked={globalShortcuts}
              disabled
              title="Not wired yet"
            >
              {#if globalShortcuts}<CheckIcon className="control-icon" />{/if}
            </Checkbox>
          </SettingRow>
        </div>

        <div class="divider"></div>

        <div class="stanza">
          <div class="eyebrow">Activation</div>
          <SettingRow top>
            <strong>Activation key</strong>
            <p>Require a modifier key to enable Harper checking in a window.</p>
            <Select
              slot="control"
              unstyled
              class="select"
              disabled
              title="Not wired yet"
              bind:value={activationKey}
            >
              <option value="off">Off</option>
              <option value="option">Option</option>
              <option value="control">Control</option>
              <option value="shift">Shift</option>
            </Select>
          </SettingRow>
        </div>
      </section>
