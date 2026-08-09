<script lang="ts">
  import { onMount } from 'svelte';
  import { ensureAccessibilityConsent } from '$lib/settings/accessibilityConsent';

  export let bundleId: string;

  let consented: boolean | null = null;
  let loading = false;

  onMount(async () => {
    if (!bundleId) {
      consented = false;
      return;
    }
    // Query native side: has user already consented?
    try {
      const already = await (await import('@tauri-apps/api/tauri')).invoke('has_user_consented', {
        bundleId,
      });
      consented = Boolean(already);
    } catch (e) {
      console.error('has_user_consented failed', e);
      consented = false;
    }
  });

  async function requestConsent() {
    loading = true;
    try {
      const ok = await ensureAccessibilityConsent(bundleId);
      consented = Boolean(ok);
    } finally {
      loading = false;
    }
  }
</script>

{#if consented === null}
  <div>Checking consent…</div>
{:else if consented}
  <div class="text-sm text-green-600">Accessibility access allowed for {bundleId}</div>
{:else}
  <button class="btn" on:click={requestConsent} disabled={loading}>
    {#if loading}Allowing…{:else}Allow accessibility access for {bundleId}{/if}
  </button>
{/if}

<style>
  .btn {
    padding: 0.5rem 1rem;
    border-radius: 6px;
    background: var(--btn-bg, #1f2937);
    color: var(--btn-fg, #fff);
    border: none;
    cursor: pointer;
  }
</style>
