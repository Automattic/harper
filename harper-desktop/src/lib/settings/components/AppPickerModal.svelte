<script lang="ts">
  export let bundleId = "";
  export let existingBundleIds: string[];
  export let isSaving = false;
  export let close: () => void;
  export let add: (bundleId: string) => void;

  $: trimmedBundleId = bundleId.trim();
  $: isDuplicate = existingBundleIds.includes(trimmedBundleId);
  $: canAdd = Boolean(trimmedBundleId) && !isDuplicate && !isSaving;

  function submit() {
    if (canAdd) {
      add(trimmedBundleId);
    }
  }
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
      <strong>Add application</strong>
      <span>Enter the app bundle ID Harper should watch.</span>
    </div>
    <div class="modal-search">
      <span class="settings-icon icon-search" aria-hidden="true"></span>
      <input
        type="text"
        placeholder="com.apple.TextEdit"
        bind:value={bundleId}
        disabled={isSaving}
        on:keydown={(event) => {
          if (event.key === "Enter") {
            submit();
          }
        }}
      />
    </div>
    <div class="modal-list">
      {#if isDuplicate}
        <div class="empty">That application is already configured.</div>
      {:else}
        <div class="empty">Use the app's macOS bundle identifier. Example: com.apple.TextEdit</div>
      {/if}
    </div>
    <div class="modal-actions">
      <button class="button" type="button" on:click={close}>Cancel</button>
      <button class="button primary" type="button" disabled={!canAdd} on:click={submit}>Add</button>
    </div>
  </div>
</div>
