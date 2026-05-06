<script lang="ts">
  import "../app.css";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  let isSettings = false;
  let isViewResolved = false;

  function hasSettingsRoute() {
    return (
      new URLSearchParams(window.location.search).get("view") === "settings" ||
      window.location.hash === "#settings"
    );
  }

  onMount(() => {
    let currentWindowLabel = "";

    try {
      currentWindowLabel = getCurrentWindow().label;
    } catch {
      currentWindowLabel = "";
    }

    isSettings = currentWindowLabel === "settings" || hasSettingsRoute();
    isViewResolved = true;

    if (isSettings) {
      document.body.classList.add("settings-view");
    }

    return () => {
      document.body.classList.remove("settings-view");
    };
  });
</script>

{#if isViewResolved}
  {#if isSettings}
    {#await import("$lib/settings/SettingsApp.svelte") then module}
      <svelte:component this={module.default} />
    {/await}
  {:else}
    {#await import("$lib/EditorView.svelte") then module}
      <svelte:component this={module.default} />
    {/await}
  {/if}
{/if}
