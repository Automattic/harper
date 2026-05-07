<script lang="ts">
  import type { AppIntegration } from "../settings-data";

  export let appSearch = "";
  export let pickerItems: AppIntegration[];
  export let close: () => void;
  export let selectApp: (app: AppIntegration) => void;
</script>

<div
  class="modal-backdrop"
  role="button"
  tabindex="0"
  aria-label="Close application picker"
  on:click={close}
  on:keydown={(event) => {
    if (event.key === "Escape" || event.key === "Enter" || event.key === " ") {
      close();
    }
  }}
>
  <div
    class="modal"
    role="dialog"
    tabindex="-1"
    aria-label="Choose an application"
    on:click|stopPropagation={() => {}}
    on:keydown|stopPropagation={(event) => {
      if (event.key === "Escape") {
        close();
      }
    }}
  >
    <div class="modal-head">
      <strong>Choose an application</strong>
      <span>Applications - /Applications</span>
    </div>
    <div class="modal-search">
      <span class="settings-icon icon-search" aria-hidden="true"></span>
      <input type="text" placeholder="Search Applications..." bind:value={appSearch} />
    </div>
    <div class="modal-list">
      {#if pickerItems.length === 0}
        <div class="empty">No applications match "{appSearch}".</div>
      {:else}
        {#each pickerItems as app}
          <button class="picker-row" type="button" on:click={() => selectApp(app)}>
            <span class="app-tile" style={`--app-tint: ${app.tint}`}>{app.name[0]}</span>
            <span class="grow">
              <strong>{app.name}</strong>
              <small>Application</small>
            </span>
            <span>{app.kind}</span>
          </button>
        {/each}
      {/if}
    </div>
    <div class="modal-actions">
      <button class="button" type="button" on:click={close}>Cancel</button>
    </div>
  </div>
</div>
