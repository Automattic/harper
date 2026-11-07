// content.js: runs inside every web page.
//
// It finds the text field you're typing in, asks the HarperMac app to check it, and draws
// underlines and a suggestion popup inside the page, so they line up exactly with the text.
//
// Reading guide, top to bottom:
//   finding fields    which elements count as "a place you write" (textarea, input, editable divs)
//   reading text      getting the text out, and mapping positions back to the page for editable divs
//   geometry          working out where each flagged word sits on screen
//   overlay           drawing the squiggles and the popup (in a shadow DOM so page styles can't touch it)
//   checking          debouncing, sending text to the app, keeping underlines steady while you type
//   frame loop        a per-frame loop that keeps underlines glued to the text as the page moves
//   popup and fixes   clicking a word and applying a replacement
//
// Text offsets everywhere are UTF-16 code units, which is what JavaScript strings and the
// HarperMac app both use, so a position from the app indexes straight into the page's text.
(() => {
  'use strict';
  if (window.__harperMac) return;
  window.__harperMac = true;

  const api = typeof browser !== 'undefined' ? browser : chrome;
  const MAX_LENGTH = 30000;
  const DEBOUNCE_MS = 300;
  const HEARTBEAT_MS = 2000;

  const state = {
    field: null,        // { el, kind: 'input' | 'editable' }
    text: '',           // the text the lints refer to
    lints: [],
    drawn: [],          // [{ lint, rect }] in viewport coordinates, for hit-testing
    seq: 0,
    timer: null,
    raf: null,
    dirty: true,
    sig: '',
    lastBeat: 0,
    disabledHere: false,
  };

  // ---------------------------------------------------------------- settings

  function loadSettings() {
    try {
      api.storage.local.get('disabledHosts').then((v) => {
        state.disabledHere = (v.disabledHosts || []).includes(location.hostname);
      }, () => {});
    } catch (_) { /* storage unavailable */ }
  }
  loadSettings();
  try {
    api.storage.onChanged.addListener(() => { loadSettings(); schedule(0); });
  } catch (_) { /* ignore */ }

  // ---------------------------------------------------------------- finding fields

  function usable(el) {
    if (el.getAttribute('spellcheck') === 'false') return false;   // code editors say so
    if (el.readOnly || el.disabled) return false;
    if (el.closest('[data-harper-ignore]')) return false;
    const ac = (el.getAttribute('autocomplete') || '').toLowerCase();
    if (/(username|email|password|one-time-code|cc-|tel|url)/.test(ac)) return false;
    return true;
  }

  function editableFor(target) {
    if (!(target instanceof Element)) return null;
    if (target instanceof HTMLTextAreaElement) return usable(target) ? { el: target, kind: 'input' } : null;
    if (target instanceof HTMLInputElement) {
      const type = (target.type || 'text').toLowerCase();
      return (type === 'text' || type === 'search') && usable(target) ? { el: target, kind: 'input' } : null;
    }
    if (target.isContentEditable) {
      let root = target;
      while (root.parentElement && root.parentElement.isContentEditable) root = root.parentElement;
      return usable(root) ? { el: root, kind: 'editable' } : null;
    }
    return null;
  }

  // ---------------------------------------------------------------- reading text

  const BLOCKS = new Set(['DIV', 'P', 'LI', 'UL', 'OL', 'H1', 'H2', 'H3', 'H4', 'H5', 'H6',
    'BLOCKQUOTE', 'PRE', 'TR', 'TABLE', 'SECTION', 'ARTICLE', 'HEADER', 'FOOTER']);
  const SKIP = new Set(['SCRIPT', 'STYLE', 'NOSCRIPT', 'TEMPLATE']);

  /** Plain text of a contenteditable plus a map from text offsets back to DOM text nodes. */
  function extract(root) {
    let text = '';
    const segs = [];
    const newline = () => { if (text && !text.endsWith('\n')) text += '\n'; };
    (function walk(node) {
      if (node.nodeType === Node.TEXT_NODE) {
        let data = node.data.replace(/[ ​﻿]/g, ' ');
        const parent = node.parentElement;
        if (!parent || !/^pre/.test(getComputedStyle(parent).whiteSpace)) data = data.replace(/[\r\n]/g, ' ');
        segs.push({ node, start: text.length, end: text.length + data.length });
        text += data;
      } else if (node.nodeType === Node.ELEMENT_NODE) {
        if (SKIP.has(node.tagName)) return;
        if (node.tagName === 'BR') { text += '\n'; return; }
        const block = BLOCKS.has(node.tagName);
        if (block) newline();
        for (const child of node.childNodes) walk(child);
        if (block) newline();
      }
    })(root);
    return { text, segs };
  }

  function snapshot(field) {
    if (field.kind === 'input') return { text: field.el.value, segs: null };
    return extract(field.el);
  }

  function locate(segs, pos, isEnd) {
    for (const s of segs) {
      if (isEnd ? (pos > s.start && pos <= s.end) : (pos >= s.start && pos < s.end)) {
        return { node: s.node, offset: pos - s.start };
      }
    }
    return null;
  }

  // ---------------------------------------------------------------- geometry

  const MIRROR_PROPS = ['fontStyle', 'fontVariant', 'fontWeight', 'fontStretch', 'fontSize', 'fontFamily',
    'lineHeight', 'letterSpacing', 'wordSpacing', 'textTransform', 'textIndent', 'textAlign', 'tabSize',
    'direction', 'wordBreak', 'overflowWrap', 'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
    'borderTopWidth', 'borderRightWidth', 'borderBottomWidth', 'borderLeftWidth', 'borderStyle'];

  /**
   * Rects for text ranges inside an <input>/<textarea>, via an invisible copy of the field.
   *
   * Browsers won't tell you where a word sits inside a textarea, but they will for ordinary
   * text. So we build a hidden <div> with the same width, fonts and padding, put the same text
   * in it, measure the words there, and shift the results onto the real field.
   */
  function measureInput(el, text, ranges) {
    const cs = getComputedStyle(el);
    const isInput = el.tagName === 'INPUT';
    const mirror = document.createElement('div');
    for (const p of MIRROR_PROPS) mirror.style[p] = cs[p];
    const bx = parseFloat(cs.borderLeftWidth) + parseFloat(cs.borderRightWidth);
    Object.assign(mirror.style, {
      position: 'fixed', left: '-100000px', top: '0', visibility: 'hidden', overflow: 'hidden',
      boxSizing: 'border-box', width: (el.clientWidth + bx) + 'px',
      whiteSpace: isInput ? 'pre' : 'pre-wrap', wordWrap: isInput ? 'normal' : 'break-word',
    });
    const textNode = document.createTextNode(text);
    mirror.appendChild(textNode);
    (document.body || document.documentElement).appendChild(mirror);

    const m = mirror.getBoundingClientRect();
    const e = el.getBoundingClientRect();
    let dx = e.left - m.left - el.scrollLeft;
    const dy = e.top - m.top - el.scrollTop;
    const contentH = el.clientHeight - parseFloat(cs.paddingTop) - parseFloat(cs.paddingBottom);

    const out = ranges.map(([start, end]) => {
      const r = document.createRange();
      try { r.setStart(textNode, start); r.setEnd(textNode, end); } catch (_) { return []; }
      return Array.from(r.getClientRects()).map((c) => {
        // Single-line inputs centre their text vertically; the mirror does not.
        const shift = isInput ? Math.max(0, (contentH - c.height) / 2) : 0;
        return { left: c.left + dx, top: c.top + dy + shift, right: c.right + dx, bottom: c.bottom + dy + shift };
      });
    });
    mirror.remove();
    return out;
  }

  function measureEditable(segs, ranges) {
    return ranges.map(([start, end]) => {
      const a = locate(segs, start, false);
      const b = locate(segs, end, true);
      if (!a || !b) return [];
      const r = document.createRange();
      try { r.setStart(a.node, a.offset); r.setEnd(b.node, b.offset); } catch (_) { return []; }
      return Array.from(r.getClientRects()).map((c) =>
        ({ left: c.left, top: c.top, right: c.right, bottom: c.bottom }));
    });
  }

  /** The visible area of the field: its box, narrowed by any clipping ancestor. */
  function visibleBox(el) {
    let box = el.getBoundingClientRect();
    box = { left: box.left, top: box.top, right: box.right, bottom: box.bottom };
    for (let p = el.parentElement; p && p !== document.documentElement; p = p.parentElement) {
      const o = getComputedStyle(p);
      if (o.overflowX === 'visible' && o.overflowY === 'visible') continue;
      const r = p.getBoundingClientRect();
      box = { left: Math.max(box.left, r.left), top: Math.max(box.top, r.top),
              right: Math.min(box.right, r.right), bottom: Math.min(box.bottom, r.bottom) };
    }
    return {
      left: Math.max(box.left, 0), top: Math.max(box.top, 0),
      right: Math.min(box.right, innerWidth), bottom: Math.min(box.bottom, innerHeight),
    };
  }

  // ---------------------------------------------------------------- overlay

  const host = document.createElement('div');
  host.setAttribute('data-harper-host', '');
  host.style.cssText = 'all:initial;position:fixed;left:0;top:0;width:0;height:0;z-index:2147483647;pointer-events:none;';
  const shadow = host.attachShadow({ mode: 'closed' });

  const wave = (color) => 'url("data:image/svg+xml,' + encodeURIComponent(
    `<svg xmlns='http://www.w3.org/2000/svg' width='6' height='3'><path d='M0 2.5 L1.5 0.5 L3 2.5 L4.5 0.5 L6 2.5' fill='none' stroke='${color}' stroke-width='1' stroke-linejoin='round'/></svg>`) + '")';

  const style = document.createElement('style');
  style.textContent = `
    :host { color-scheme: light dark; }
    .u { position: fixed; height: 3px; background-repeat: repeat-x; background-size: 6px 3px; pointer-events: none; }
    .u.spell { background-image: ${wave('#e5484d')}; }
    .u.grammar { background-image: ${wave('#3b82f6')}; }
    .pop { position: fixed; min-width: 160px; max-width: 300px; padding: 10px 12px; border-radius: 10px;
      font: 13px/1.35 -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; pointer-events: auto;
      background: #fff; color: #1d1d1f; border: 1px solid rgba(0,0,0,.12); box-shadow: 0 8px 28px rgba(0,0,0,.22); }
    .pop .msg { font-size: 12px; color: #6e6e73; margin-bottom: 6px; }
    .pop button { display: block; width: 100%; text-align: left; border: 0; background: none; color: inherit;
      font: inherit; padding: 4px 6px; margin: 0; border-radius: 6px; cursor: pointer; }
    .pop button:hover { background: rgba(0,0,0,.07); }
    .pop button.fix { font-weight: 600; }
    .pop .more { margin-top: 6px; padding-top: 6px; border-top: 1px solid rgba(0,0,0,.1); }
    .pop .more button { font-size: 11px; color: #6e6e73; }
    @media (prefers-color-scheme: dark) {
      .pop { background: #2c2c2e; color: #f5f5f7; border-color: rgba(255,255,255,.14); }
      .pop .msg, .pop .more button { color: #a1a1a6; }
      .pop button:hover { background: rgba(255,255,255,.1); }
      .pop .more { border-color: rgba(255,255,255,.12); }
    }`;
  shadow.appendChild(style);
  const layer = document.createElement('div');
  shadow.appendChild(layer);

  function mountHost() {
    if (!host.isConnected && document.documentElement) document.documentElement.appendChild(host);
  }

  let popup = null;
  function closePopup() { if (popup) { popup.remove(); popup = null; } }

  function draw() {
    const f = state.field;
    layer.querySelectorAll('.u').forEach((n) => n.remove());
    state.drawn = [];
    if (!f || state.disabledHere || !state.lints.length) return;
    mountHost();

    const ranges = state.lints.map((l) => [l.start, l.end]);
    let measured;
    if (f.kind === 'input') measured = measureInput(f.el, f.el.value, ranges);
    else measured = measureEditable(f.segs || extract(f.el).segs, ranges);

    const box = visibleBox(f.el);
    const seen = new Set();
    state.lints.forEach((lint, i) => {
      for (const r of measured[i] || []) {
        const left = Math.max(r.left, box.left), right = Math.min(r.right, box.right);
        if (right - left < 2 || r.bottom < box.top || r.bottom > box.bottom + 1) continue;
        const key = [lint.start, Math.round(left), Math.round(r.top)].join(',');
        if (seen.has(key)) continue;
        seen.add(key);
        const u = document.createElement('div');
        u.className = 'u ' + (lint.kind === 'SpellCheck' ? 'spell' : 'grammar');
        u.style.left = left + 'px';
        u.style.top = (r.bottom - 3) + 'px';
        u.style.width = (right - left) + 'px';
        layer.appendChild(u);
        state.drawn.push({ lint, rect: { left, right, top: r.top, bottom: r.bottom } });
      }
    });
  }

  function layoutSignature(f) {
    const r = f.el.getBoundingClientRect();
    return [r.left, r.top, r.width, r.height, f.el.scrollLeft, f.el.scrollTop, innerWidth, innerHeight].join('|');
  }

  // ---------------------------------------------------------------- checking

  function carryOver(lints, oldText, newText) {
    let p = 0;
    while (p < oldText.length && p < newText.length && oldText[p] === newText[p]) p++;
    let s = 0;
    while (s < oldText.length - p && s < newText.length - p &&
           oldText[oldText.length - 1 - s] === newText[newText.length - 1 - s]) s++;
    const oldEnd = oldText.length - s;
    const delta = newText.length - oldText.length;
    const out = [];
    for (const l of lints) {
      if (l.end < p) out.push(l);
      else if (l.start > oldEnd) out.push({ ...l, start: l.start + delta, end: l.end + delta });
    }
    return out;
  }

  function schedule(delay) {
    clearTimeout(state.timer);
    state.timer = setTimeout(runCheck, delay === undefined ? DEBOUNCE_MS : delay);
  }

  function clearLints() {
    state.lints = [];
    state.dirty = true;
  }

  async function runCheck() {
    const f = state.field;
    if (!f || state.disabledHere) { clearLints(); return; }
    const snap = snapshot(f);
    f.segs = snap.segs;
    state.text = snap.text;
    const trimmed = snap.text.trim();
    const urlLike = !/\s/.test(trimmed) && /[\/.@:]/.test(trimmed);
    if (!trimmed || snap.text.length > MAX_LENGTH || urlLike) { clearLints(); return; }

    const mine = ++state.seq;
    let res = null;
    try { res = await api.runtime.sendMessage({ type: 'check', text: snap.text }); } catch (_) { /* app not running */ }
    if (mine !== state.seq || state.field !== f) return;
    if (!res || !res.ok || !res.enabled) { clearLints(); return; }

    const now = snapshot(f);
    f.segs = now.segs;
    // If the user kept typing while we waited, shift the results onto the new text.
    state.lints = (now.text === snap.text ? res.lints : carryOver(res.lints, snap.text, now.text))
      .filter((l) => l.start >= 0 && l.end > l.start && l.end <= now.text.length);
    state.text = now.text;
    if (now.text !== snap.text) schedule();
    state.dirty = true;
  }

  // ---------------------------------------------------------------- frame loop

  function tick() {
    state.raf = null;
    const f = state.field;
    if (!f || !f.el.isConnected) { detach(); return; }

    const now = Date.now();
    // Only while the field actually has focus, so the app takes over for anything we don't cover.
    const focused = f.el.matches(':focus') || f.el.matches(':focus-within');
    if (focused && now - state.lastBeat > HEARTBEAT_MS && document.hasFocus()) {
      state.lastBeat = now;
      try { api.runtime.sendMessage({ type: 'handling' }).catch(() => {}); } catch (_) { /* ignore */ }
    }

    // Programmatic value changes (React and friends) don't always fire `input`.
    if (f.kind === 'input' && f.el.value !== state.text) onTextChanged();

    const sig = layoutSignature(f);
    if (state.dirty || sig !== state.sig) {
      state.sig = sig;
      state.dirty = false;
      draw();
    }
    state.raf = requestAnimationFrame(tick);
  }

  function onTextChanged() {
    const f = state.field;
    if (!f) return;
    closePopup();
    const snap = snapshot(f);
    f.segs = snap.segs;
    if (snap.text !== state.text) {
      state.lints = carryOver(state.lints, state.text, snap.text);
      state.text = snap.text;
    }
    state.dirty = true;
    schedule();
  }

  // ---------------------------------------------------------------- attach / detach

  let observer = null;

  function attach(found) {
    if (state.field && state.field.el === found.el) return;
    detach();
    state.field = found;
    state.text = '';
    state.lints = [];
    state.sig = '';
    state.dirty = true;
    state.lastBeat = 0;
    if (found.kind === 'editable') {
      observer = new MutationObserver(onTextChanged);
      observer.observe(found.el, { childList: true, characterData: true, subtree: true });
    }
    schedule(0);
    if (!state.raf) state.raf = requestAnimationFrame(tick);
  }

  function detach() {
    closePopup();
    if (observer) { observer.disconnect(); observer = null; }
    clearTimeout(state.timer);
    state.seq++;
    state.field = null;
    state.lints = [];
    state.drawn = [];
    if (state.raf) { cancelAnimationFrame(state.raf); state.raf = null; }
    layer.querySelectorAll('.u').forEach((n) => n.remove());
  }

  document.addEventListener('focusin', (e) => {
    const target = e.composedPath ? e.composedPath()[0] : e.target;
    const found = editableFor(target);
    if (found) attach(found); else detach();
  }, true);

  document.addEventListener('input', (e) => {
    if (state.field && (e.target === state.field.el || state.field.el.contains(e.target))) onTextChanged();
  }, true);

  window.addEventListener('blur', () => { if (!document.hasFocus()) detach(); });
  window.addEventListener('scroll', () => { state.dirty = true; closePopup(); }, true);
  window.addEventListener('resize', () => { state.dirty = true; });

  if (document.activeElement) {
    const found = editableFor(document.activeElement);
    if (found) attach(found);
  }

  // Test hook: only exists if the page opts in with <html data-harper-debug>.
  if (document.documentElement.hasAttribute('data-harper-debug')) {
    window.__harperMacDebug = () => ({
      field: !!state.field, lints: state.lints.length, drawn: state.drawn.length, popup: !!popup, text: state.text.length,
    });
  }

  // ---------------------------------------------------------------- popup and fixes

  function hit(x, y) {
    return state.drawn.find(({ rect }) =>
      x >= rect.left - 2 && x <= rect.right + 2 && y >= rect.top - 2 && y <= rect.bottom + 3);
  }

  document.addEventListener('mouseup', (e) => {
    if (e.button !== 0 || !state.field || state.disabledHere) return;
    if (e.composedPath().includes(host)) return;
    const h = hit(e.clientX, e.clientY);
    if (h) openPopup(h); else closePopup();
  }, true);

  document.addEventListener('keydown', (e) => { if (e.key === 'Escape') closePopup(); }, true);

  function button(label, className, onClick) {
    const b = document.createElement('button');
    b.textContent = label;
    if (className) b.className = className;
    // Keep keyboard focus (and the selection) in the page's field.
    b.addEventListener('mousedown', (e) => e.preventDefault());
    b.addEventListener('click', (e) => { e.preventDefault(); e.stopPropagation(); onClick(); });
    return b;
  }

  function openPopup({ lint, rect }) {
    closePopup();
    const pop = document.createElement('div');
    pop.className = 'pop';
    const msg = document.createElement('div');
    msg.className = 'msg';
    msg.textContent = lint.message;
    pop.appendChild(msg);

    for (const fix of lint.fixes) {
      pop.appendChild(button(fix.text === '' ? fix.label : fix.text, 'fix', () => applyFix(lint, fix.text)));
    }
    if (lint.commands.length) {
      const more = document.createElement('div');
      more.className = 'more';
      for (const c of lint.commands) {
        more.appendChild(button(c.kind === 'ignore' ? 'Ignore' : c.label, '', () => runCommand(c)));
      }
      pop.appendChild(more);
    }

    layer.appendChild(pop);
    const w = pop.offsetWidth, h = pop.offsetHeight;
    let top = rect.bottom + 6;
    if (top + h > innerHeight - 4) top = Math.max(4, rect.top - h - 6);
    pop.style.left = Math.max(4, Math.min(rect.left, innerWidth - w - 4)) + 'px';
    pop.style.top = top + 'px';
    popup = pop;
  }

  function applyFix(lint, replacement) {
    const f = state.field;
    closePopup();
    if (!f) return;
    const snap = snapshot(f);
    if (snap.text !== state.text || snap.text.slice(lint.start, lint.end) === '') { schedule(0); return; }

    const before = snap.text;
    f.el.focus({ preventScroll: true });
    if (f.kind === 'input') {
      f.el.setSelectionRange(lint.start, lint.end);
    } else {
      const a = locate(snap.segs, lint.start, false), b = locate(snap.segs, lint.end, true);
      if (!a || !b) return;
      const range = document.createRange();
      range.setStart(a.node, a.offset);
      range.setEnd(b.node, b.offset);
      const sel = getSelection();
      sel.removeAllRanges();
      sel.addRange(range);
    }
    // execCommand keeps the page's undo stack and fires the events frameworks listen for.
    let ok = replacement === '' ? document.execCommand('delete') : document.execCommand('insertText', false, replacement);
    if (f.kind === 'input' && (!ok || f.el.value === before)) {
      f.el.setRangeText(replacement, lint.start, lint.end, 'end');
      f.el.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText', data: replacement }));
    }
  }

  async function runCommand(c) {
    closePopup();
    try { await api.runtime.sendMessage({ type: 'command', command: c.command, arguments: c.arguments }); } catch (_) {}
    // Dictionary and ignore changes only show once the text is checked again.
    setTimeout(() => schedule(0), 250);
  }
})();
