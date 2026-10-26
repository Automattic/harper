// Relays messages from pages to the HarperMac app, which listens on loopback only.
// Content scripts can't call it directly: pages would hit CORS and local-network prompts.

const api = typeof browser !== 'undefined' ? browser : chrome;
const BASE = 'http://127.0.0.1:47321';
const FAMILY = /Chrome\//.test(navigator.userAgent) ? 'chromium' : 'safari';

async function post(path, body) {
  const res = await fetch(BASE + path, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'X-HarperMac': '1' },
    body: JSON.stringify(body || {}),
  });
  if (!res.ok) throw new Error('HTTP ' + res.status);
  return res.json();
}

async function handle(msg) {
  switch (msg && msg.type) {
    case 'check': {
      const r = await post('/check', { text: msg.text });
      return { ok: true, enabled: r.enabled !== false, lints: r.lints || [] };
    }
    case 'command':
      await post('/command', { command: msg.command, arguments: msg.arguments });
      return { ok: true };
    case 'handling':
      // Tells the app a field is being handled here, so it doesn't draw its own underlines.
      await post('/ping', { family: FAMILY, handling: true });
      return { ok: true };
    case 'status': {
      const r = await post('/ping', { family: FAMILY });
      return { ok: true, enabled: r.enabled, dialect: r.dialect };
    }
    default:
      return { ok: false, error: 'unknown message' };
  }
}

api.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
  handle(msg).then(sendResponse, (e) => sendResponse({ ok: false, error: String(e) }));
  return true; // keep the channel open for the async reply
});
