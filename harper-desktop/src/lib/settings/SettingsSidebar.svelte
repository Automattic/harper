<script lang="ts">
import { FOOTER_NAV_ITEMS, MAIN_NAV_ITEMS, type SectionId } from './settings-data';

export let active: SectionId;
export let hasSetupAlert = false;
</script>

<nav class="sidebar" aria-label="Settings sections">
  <div class="group">
    {#each MAIN_NAV_ITEMS as item}
      <button
        type="button"
        class:selected={active === item.id}
        class="nav-item"
        on:click={() => (active = item.id)}
      >
        <span class="tile" style={`--tile-gradient: ${item.gradient}`}>
          <span class={`settings-icon ${item.id}`} aria-hidden="true"></span>
        </span>
        <span class="label">{item.label}</span>
        {#if item.id === "getting-started" && hasSetupAlert}
          <span class="alert" aria-label="Action needed"></span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="separator"></div>

  <div class="group">
    {#each FOOTER_NAV_ITEMS as item}
      <button
        type="button"
        class:selected={active === item.id}
        class="nav-item"
        on:click={() => (active = item.id)}
      >
        <span class="tile" style={`--tile-gradient: ${item.gradient}`}>
          <span class={`settings-icon ${item.id}`} aria-hidden="true"></span>
        </span>
        <span class="label">{item.label}</span>
      </button>
    {/each}
  </div>
</nav>

<style>
  .sidebar {
    width: 190px;
    flex: 0 0 190px;
    background: var(--settings-bg-sidebar);
    border-right: 0.5px solid var(--settings-line-strong);
    display: flex;
    flex-direction: column;
    padding: 18px 8px 12px;
  }

  .group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .separator {
    height: 1px;
    margin: 14px 10px 12px;
    background: var(--settings-line);
  }

  .nav-item {
    width: 100%;
    height: 30px;
    display: flex;
    align-items: center;
    gap: 10px;
    border: 0;
    border-radius: 6px;
    padding: 0 10px;
    background: transparent;
    color: var(--settings-ink);
    font: inherit;
    font-size: 13px;
    text-align: left;
    cursor: default;
  }

  .nav-item:hover {
    background: rgba(0, 0, 0, 0.045);
  }

  .nav-item.selected {
    background: var(--settings-accent);
    color: #fff;
  }

  .tile {
    width: 22px;
    height: 22px;
    flex: 0 0 22px;
    display: grid;
    place-items: center;
    border-radius: 6px;
    background: var(--tile-gradient);
    color: #fff;
    box-shadow:
      0 1px 1px rgba(0, 0, 0, 0.08),
      inset 0 0.5px 0 rgba(255, 255, 255, 0.35);
  }

  .tile :global(.settings-icon) {
    transform: scale(0.78);
  }

  .label {
    min-width: 0;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .alert {
    width: 7px;
    height: 7px;
    flex: 0 0 7px;
    border-radius: 999px;
    background: #d93920;
    box-shadow: 0 0 0 2px rgba(217, 57, 32, 0.18);
  }

  .selected .alert {
    background: #fff;
    box-shadow: none;
  }
</style>
