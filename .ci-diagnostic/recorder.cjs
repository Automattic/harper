'use strict';

// This function is serialized into each frame. Keep every dependency inside it.
function installPassiveRecorder() {
  if (window.__harperCIDiagnostic) return;
  const CAPACITY = 2048, TEXT_LIMIT = 2048, STACK_LIMIT = 1800;
  const ring = new Array(CAPACITY), identities = new WeakMap(), listeners = new WeakMap();
  const registrations = [];
  let total = 0, identity = 0, invocation = 0, listenerIdentity = 0, diagnosticErrors = 0;
  let rootCache = null;
  const child = window !== window.top;
  const add = EventTarget.prototype.addEventListener, remove = EventTarget.prototype.removeEventListener;
  const enqueue = window.queueMicrotask.bind(window);
  const now = performance.now.bind(performance);
  const clockOrigin = performance.timeOrigin;
  const clip = (value, length = TEXT_LIMIT) => typeof value === 'string' ? value.slice(0, length) : value;
  const stack = () => clip(new Error().stack || '', STACK_LIMIT);
  function id(value) {
    if (!value || (typeof value !== 'object' && typeof value !== 'function')) return null;
    if (!identities.has(value)) identities.set(value, ++identity);
    return identities.get(value);
  }
  function node(value) {
    if (!value) return null;
    if (!value.nodeType) return {id:id(value), name:value === window ? 'window' : 'object'};
    const parts = [];
    let at = value, depth = 0;
    while (at && depth++ < 12) {
      let index = 0, sibling = at.previousSibling;
      while (sibling && index < 200) {index++; sibling = sibling.previousSibling;}
      parts.push(`${at.nodeName}${at.id ? '#' + clip(at.id, 80) : ''}[${index}${sibling ? '+' : ''}]`);
      at = at.parentNode || (at.host || null);
    }
    const text = value.nodeType === 3 ? value.nodeValue : null;
    return {id:id(value), name:value.nodeName, path:clip(parts.reverse().join('/'),700),
      pathTruncated:!!at, connected:value.isConnected, text:clip(text,256), textLength:text?.length ?? null};
  }
  function root() {
    if (!rootCache?.isConnected) rootCache = document.querySelector('[data-lexical-editor="true"]');
    return rootCache;
  }
  function detailed() {return child || !!root();}
  function model(state) {
    if (!state) return null;
    const selection = state._selection;
    const point = p => p ? {key:p.key, offset:p.offset, type:p.type} : null;
    return {readOnly:state._readOnly, selection:selection ? {
      anchor:point(selection.anchor), focus:point(selection.focus), dirty:selection.dirty,
      format:selection.format, style:clip(selection.style,256), type:selection.constructor?.name
    } : null};
  }
  function points(value) {
    return value ? {anchor:{node:node(value.anchorNode),offset:value.anchorOffset},
      focus:{node:node(value.focusNode),offset:value.focusOffset}} : null;
  }
  function snapshot() {
    const focused = [];
    let active = document.activeElement;
    for (let i = 0; active && i < 8; i++) {focused.push(node(active)); active = active.shadowRoot?.activeElement;}
    const out = {visibility:document.visibilityState, hasFocus:document.hasFocus(), focused};
    const el = root();
    if (!child && !el) return out;
    const s = document.getSelection();
    out.dom = s ? {...points(s), rangeCount:s.rangeCount, type:s.type, direction:s.direction} : null;
    if (el) {
      out.root = node(el);out.text = clip(el.textContent);out.textLength = el.textContent.length;
      const ed = el.__lexicalEditor;
      if (ed) {
        const input = ed._inputState, applied = input?.selectionChangeFromDOMUpdatePoints;
        const interior = (n,o) => !!n && n.nodeType === 3 && o !== 0 && o !== n.nodeValue.length;
        const sameApplied = !!s && !!applied && applied.anchorNode === s.anchorNode && applied.anchorOffset === s.anchorOffset && applied.focusNode === s.focusNode && applied.focusOffset === s.focusOffset;
        out.lexical = {editor:id(ed), committed:model(ed._editorState), pending:model(ed._pendingEditorState),
          input:input ? {isSelectionChangeFromDOMUpdate:input.isSelectionChangeFromDOMUpdate,
            isSelectionChangeFromMouseDown:input.isSelectionChangeFromMouseDown,
            postDeleteSelectionToRestore:!!input.postDeleteSelectionToRestore,
            applied:points(applied), appliedIsNull:applied === null,
            // A raw-DOM diagnostic predicate, not a claim that the host took this branch.
            rawDOMSkipCandidate:!!(s && input.isSelectionChangeFromDOMUpdate && interior(s.anchorNode,s.anchorOffset) && interior(s.focusNode,s.focusOffset) && !input.postDeleteSelectionToRestore && (applied === null || sameApplied)),
            sameApplied} : null};
      } else out.lexical = null;
    }
    return out;
  }
  function errorInfo(error) {
    try {return {name:clip(error?.name,100),message:clip(error?.message || String(error),500),stack:clip(error?.stack,STACK_LIMIT)};}
    catch {return {message:'unreadable thrown value'};}
  }
  function record(kind, extra = {}, withState = true) {
    // Observation failures must never replace the application's exception.
    try {
      const event = {seq:total + 1, t:now(), kind, ...extra};
      if (withState) event.state = snapshot();
      ring[total % CAPACITY] = event;
      total++;
    } catch {diagnosticErrors++;}
  }
  function eventData(event, phase) {
    return {type:event.type, phase, eventPhase:event.eventPhase, trusted:event.isTrusted,
      timeStamp:event.timeStamp, key:event.key, code:event.code, repeat:event.repeat,
      inputType:event.inputType, data:clip(event.data,256), button:event.button,
      pointerType:event.pointerType, x:event.clientX, y:event.clientY,
      target:node(event.target), relatedTarget:node(event.relatedTarget), defaultPrevented:event.defaultPrevented};
  }
  function observe(event, phase) {
    try {record('event', {event:eventData(event,phase)});} catch {diagnosticErrors++;}
  }
  const types = ['keydown','keyup','selectionchange','beforeinput','input','focus','blur','focusin','focusout','pointerdown','pointerup','pointerover','pointerout','click'];
  for (const type of types) {
    add.call(document,type,event=>observe(event,'capture'),{capture:true,passive:true});
    add.call(document,type,event=>observe(event,'bubble'),{passive:true});
  }
  for (const type of ['focus','blur','visibilitychange','pagehide','pageshow']) {
    add.call(window,type,event=>{if(event.target===window || event.target===document)observe(event,'window');},{capture:true,passive:true});
  }
  add.call(window,'error',event=>record('error',{error:errorInfo(event.error || event.message)}),{passive:true});
  add.call(window,'unhandledrejection',event=>record('unhandledrejection',{error:errorInfo(event.reason)}),{passive:true});

  // One stable wrapper per listener, independent of capture/options. Native code
  // still owns duplicate detection, capture, once removal, AbortSignal and getters.
  // All document selectionchange listeners use this mapping so a later source-
  // matched registration cannot create a second identity for the same callback.
  EventTarget.prototype.addEventListener = function(type, listener, options) {
    if (this !== document || type !== 'selectionchange' || !listener || (typeof listener !== 'function' && typeof listener !== 'object')) return Reflect.apply(add,this,arguments);
    let entry = listeners.get(listener);
    if (!entry) {
      entry = {id:++listenerIdentity, lexical:false, stack:null, wrapper:null};
      entry.wrapper = function(event) {
        const called = ++invocation;
        const traced = entry.lexical;
        if (traced) {try {record('lexical:before',{listener:entry.id,call:called,event:eventData(event,'callback')});} catch {diagnosticErrors++;}}
        try {
          return typeof listener === 'function' ? Reflect.apply(listener,this,arguments) : Reflect.apply(listener.handleEvent,listener,arguments);
        } catch(error) {
          if(traced) record('lexical:throw',{listener:entry.id,call:called,error:errorInfo(error)});
          throw error;
        } finally {
          if(traced) {
            record('lexical:after',{listener:entry.id,call:called});
            try {enqueue(()=>record('lexical:microtask',{listener:entry.id,call:called}));} catch {diagnosticErrors++;}
          }
        }
      };
      listeners.set(listener,entry);
    }
    try {
      const source = stack();
      if (/\blexical@[^\s/]+\//i.test(source)) {
        entry.lexical = true;
        if (!entry.stack) {
          entry.stack = source;
          if(registrations.length < 32) registrations.push({listener:entry.id,stack:source});
        }
        record('lexical:register',{listener:entry.id,stack:source},false);
      }
    } catch {diagnosticErrors++;}
    return add.call(this,type,entry.wrapper,options);
  };
  EventTarget.prototype.removeEventListener = function(type,listener,options) {
    const entry = this === document && type === 'selectionchange' && listener && (typeof listener==='function'||typeof listener==='object') ? listeners.get(listener) : null;
    return entry ? remove.call(this,type,entry.wrapper,options) : Reflect.apply(remove,this,arguments);
  };

  if (typeof window.requestAnimationFrame === 'function') {
    const raf = window.requestAnimationFrame;
    window.requestAnimationFrame = function(callback) {
      if(typeof callback !== 'function') return Reflect.apply(raf,this,arguments);
      let source = null;
      try {if(detailed()) source = stack();} catch {diagnosticErrors++;}
      let handle;
      handle = raf.call(this,function(timestamp) {
        let trace = false;
        try {trace = detailed();} catch {diagnosticErrors++;}
        if(trace) record('raf:before',{handle,timestamp,stack:source});
        try {return Reflect.apply(callback,this,arguments);}
        catch(error) {if(trace)record('raf:throw',{handle,error:errorInfo(error)});throw error;}
        finally {if(trace)record('raf:after',{handle,timestamp,stack:source});}
      });
      return handle;
    };
  }
  if (typeof Selection !== 'undefined') {
    const writers = ['setBaseAndExtent','collapse','setPosition','collapseToStart','collapseToEnd','extend','modify','selectAllChildren','addRange','removeRange','removeAllRanges','empty','deleteFromDocument'];
    for(const method of writers) {
      const descriptor = Object.getOwnPropertyDescriptor(Selection.prototype,method);
      if(typeof descriptor?.value !== 'function') continue;
      const original = descriptor.value;
      Object.defineProperty(Selection.prototype,method,{...descriptor,value:function(...args) {
        let trace = false, called, source;
        try {trace = detailed();} catch {diagnosticErrors++;}
        if(trace) {
          called = ++invocation;
          try {
            source = stack();
            const values = args.map(value=>value instanceof Node ? node(value) : (value instanceof Range ? {start:node(value.startContainer),startOffset:value.startOffset,end:node(value.endContainer),endOffset:value.endOffset} : (typeof value==='object' ? {id:id(value)} : clip(value,256))));
            record('selection:before',{method,call:called,stack:source,args:values});
          } catch {diagnosticErrors++;}
        }
        try {return Reflect.apply(original,this,args);}
        catch(error) {if(trace)record('selection:throw',{method,call:called,error:errorInfo(error)});throw error;}
        finally {if(trace)record('selection:after',{method,call:called});}
      }});
    }
  }

  // Only child documents observe popup mutations. Layout/style reads occur once
  // per affected popup per delivered mutation batch, never on keyboard events.
  if(child && typeof MutationObserver !== 'undefined') {
    const selector = 'harper-render-box,.harper-popup,.harper-container,.harper-flyout,[data-harper-popover]';
    const shadowObservers = new Map(), popupHosts = new WeakMap();
    const mutationOptions = {subtree:true,childList:true,attributes:true,attributeOldValue:true,attributeFilter:['style','class','hidden','open','popover']};
    function candidates(value) {
      if(value.nodeType!==1)return [];
      const found=value.matches(selector)?[value]:[];
      for(const match of value.querySelectorAll(selector)) {found.push(match);if(found.length>=32)break;}
      for(let i=0;i<found.length&&i<32;i++) {
        const candidate=found[i];
        const owner=candidate.getRootNode().host;
        if(owner)popupHosts.set(candidate,owner);
        if(candidate.localName!=='harper-render-box'||!candidate.shadowRoot)continue;
        if(!shadowObservers.has(candidate)&&shadowObservers.size<32&&candidate.isConnected) {
          const shadowObserver=new MutationObserver(onMutations);
          shadowObserver.observe(candidate.shadowRoot,mutationOptions);
          shadowObservers.set(candidate,shadowObserver);
        }
        for(const nested of candidate.shadowRoot.querySelectorAll(selector)) {
          if(found.length>=32)break;
          popupHosts.set(nested,candidate);found.push(nested);
        }
      }
      return found;
    }
    function onMutations(records) {
      try {
        const affected = new Map();
        function collect(value,action,attribute,oldValue) {
          for(const candidate of candidates(value)) {
            if(affected.size>=64&&!affected.has(candidate))continue;
            const entry=affected.get(candidate)||{actions:[]};
            if(entry.actions.length<16)entry.actions.push({action,attribute,oldValue:clip(oldValue,500)});
            affected.set(candidate,entry);
          }
        }
        for(const mutation of records) {
          if(mutation.type==='childList') {
            for(const value of mutation.addedNodes)collect(value,'add');
            for(const value of mutation.removedNodes)collect(value,'remove');
          } else collect(mutation.target,'attribute',mutation.attributeName,mutation.oldValue);
        }
        for(const [element,entry] of affected) {
          const style=element.isConnected?getComputedStyle(element):null;
          const rect=element.isConnected?element.getBoundingClientRect():null;
          for(const change of entry.actions)record('popup',{...change,node:node(element),shadowHost:node(popupHosts.get(element)),popoverOpen:element.matches(':popover-open'),className:clip(element.className,300),inlineStyle:clip(element.getAttribute('style'),700),hidden:element.hidden,
            computed:style?{display:style.display,visibility:style.visibility,opacity:style.opacity,position:style.position}:null,
            box:rect?{x:rect.x,y:rect.y,width:rect.width,height:rect.height}:null});
        }
      } catch {diagnosticErrors++;}
      finally {
        for(const [host,shadowObserver] of shadowObservers)if(!host.isConnected){shadowObserver.disconnect();shadowObservers.delete(host);}
      }
    }
    const observer = new MutationObserver(onMutations);
    observer.observe(document,mutationOptions);
  }
  Object.defineProperty(window,'__harperCIDiagnostic',{configurable:true,value:Object.freeze({
    export() {
      const events = [], count = Math.min(total,CAPACITY), first = total - count;
      for(let i=first;i<total;i++)events.push(ring[i%CAPACITY]);
      return {version:1,capacity:CAPACITY,total,dropped:first,diagnosticErrors,timeOrigin:clockOrigin,
        frame:{child,url:clip(location.href,2000),name:clip(window.name,200)},registrations:registrations.slice(),events};
    }
  })});
  record('installed',{},false);
}
module.exports = { installPassiveRecorder };
