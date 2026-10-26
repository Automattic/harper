// The little window that opens when you click the toolbar button. It shows whether the HarperMac
// app is running, and lets you switch checking on or off for the site you're looking at.

// Chrome calls its extension API `chrome`, Safari calls it `browser`; use whichever exists.
const api = typeof browser !== 'undefined' ? browser : chrome;
const $ = (id) => document.getElementById(id);

async function showStatus() {
  let res = null;
  try { res = await api.runtime.sendMessage({ type: 'status' }); } catch (_) { /* ignore */ }
  if (res && res.ok) {
    $('dot').className = 'dot ok';
    $('status').textContent = res.enabled
      ? `Connected · ${res.dialect} spelling`
      : 'Connected, but checking is switched off in the app';
  } else {
    $('dot').className = 'dot bad';
    $('status').textContent = 'HarperMac app not running';
    $('hint').hidden = false;
  }
}

async function showSiteToggle() {
  const [tab] = await api.tabs.query({ active: true, currentWindow: true });
  let host = '';
  try { host = new URL(tab.url).hostname; } catch (_) { /* no url */ }
  if (!host) return;
  const { disabledHosts = [] } = await api.storage.local.get('disabledHosts');
  $('siteRow').hidden = false;
  $('siteLabel').textContent = `Check on ${host}`;
  $('site').checked = !disabledHosts.includes(host);
  $('site').addEventListener('change', async () => {
    const stored = (await api.storage.local.get('disabledHosts')).disabledHosts || [];
    const next = stored.filter((h) => h !== host);
    if (!$('site').checked) next.push(host);
    await api.storage.local.set({ disabledHosts: next });
  });
}

showStatus();
showSiteToggle().catch(() => {});
