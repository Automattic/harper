import {
	BinaryModule,
	type Dialect,
	type LintConfig,
	LocalLinter,
	unpackWeirpackBytes,
} from 'harper.js';
import { type UnpackedLintGroups, unpackLint } from 'lint-framework';
import type { PopupState } from '../PopupState';
import {
	ActivationKey,
	type AddToUserDictionaryRequest,
	type AddWeirpackRequest,
	createUnitResponse,
	type GetActivationKeyResponse,
	type GetConfigRequest,
	type GetConfigResponse,
	type GetDefaultStatusResponse,
	type GetDialectRequest,
	type GetDialectResponse,
	type GetDomainStatusRequest,
	type GetDomainStatusResponse,
	type GetEnabledDomainsResponse,
	type GetHotkeyResponse,
	type GetInstalledOnRequest,
	type GetInstalledOnResponse,
	type GetLintDescriptionsRequest,
	type GetLintDescriptionsResponse,
	type GetReviewedRequest,
	type GetReviewedResponse,
	type GetUserDictionaryResponse,
	type GetWeirpacksResponse,
	type Hotkey,
	type IgnoreLintRequest,
	type LintRequest,
	type LintResponse,
	type OpenReportErrorRequest,
	type PostFormDataRequest,
	type PostFormDataResponse,
	type RemoveWeirpackRequest,
	type Request,
	type Response,
	type SetActivationKeyRequest,
	type SetConfigRequest,
	type SetDefaultStatusRequest,
	type SetDialectRequest,
	type SetDomainStatusRequest,
	type SetHotkeyRequest,
	type SetReviewedRequest,
	type SetUserDictionaryRequest,
	type UnitResponse,
	type WeirpackMeta,
} from '../protocol';
import { detectBrowserDialect } from './detectDialect';

console.log('background is running');

chrome.runtime.onInstalled.addListener((details) => {
	if (details.reason === chrome.runtime.OnInstalledReason.INSTALL) {
		chrome.runtime.setUninstallURL('https://writewithharper.com/uninstall-browser-extension');
		chrome.tabs.create({ url: 'https://writewithharper.com/install-browser-extension' });
	}
});

chrome.runtime.onMessage.addListener((request, _sender, sendResponse) => {
	handleRequest(request).then(sendResponse);

	return true;
});

let linter: LocalLinter;
const WEIRPACKS_KEY = 'weirpacks';

getDialect()
	.then(setDialect)
	.catch((err) => console.error('Failed to initialize linter:', err));
setInstalledOnIfMissing();

async function enableDefaultDomains() {
	const defaultEnabledDomains = [
		'old.reddit.com',
		'sh.reddit.com',
		'www.reddit.com',
		'chatgpt.com',
		'www.perplexity.ai',
		'textarea.online',
		'webmail.porkbun.com',
		'mail.google.com',
		'trix-editor.org',
		'github.com',
		'messages.google.com',
		'blank.page',
		'blankpage.im',
		'froala.com',
		'playground.lexical.dev',
		'discord.com',
		'www.youtube.com',
		'www.instagram.com',
		'web.whatsapp.com',
		'outlook.live.com',
		'www.linkedin.com',
		'bsky.app',
		'pootlewriter.com',
		'www.tumblr.com',
		'dayone.me',
		'medium.com',
		'x.com',
		'www.notion.so',
		'hashnode.com',
		'www.slatejs.org',
		'localhost',
		'writewithharper.com',
		'prosemirror.net',
		'draftjs.org',
		'gitlab.com',
		'core.trac.wordpress.org',
		'write.ellipsus.com',
		'www.facebook.com',
		'www.upwork.com',
		'news.ycombinator.com',
		'classroom.google.com',
		'quilljs.com',
		'www.wattpad.com',
		'ckeditor.com',
		'app.slack.com',
		'openrouter.ai',
	];

	for (const item of defaultEnabledDomains) {
		if (!(await isDomainSet(item))) {
			setDomainEnable(item, true);
		}
	}
}

enableDefaultDomains();

function handleRequest(message: Request): Promise<Response> {
	console.log(`Handling ${message.kind} request`);

	switch (message.kind) {
		case 'lint':
			return handleLint(message);
		case 'getConfig':
			return handleGetConfig(message);
		case 'setConfig':
			return handleSetConfig(message);
		case 'getLintDescriptions':
			return handleGetLintDescriptions(message);
		case 'setDialect':
			return handleSetDialect(message);
		case 'getDialect':
			return handleGetDialect(message);
		case 'getDomainStatus':
			return handleGetDomainStatus(message);
		case 'setDomainStatus':
			return handleSetDomainStatus(message);
		case 'addToUserDictionary':
			return handleAddToUserDictionary(message);
		case 'ignoreLint':
			return handleIgnoreLint(message);
		case 'setDefaultStatus':
			return handleSetDefaultStatus(message);
		case 'getDefaultStatus':
			return handleGetDefaultStatus();
		case 'getEnabledDomains':
			return handleGetEnabledDomains();
		case 'getUserDictionary':
			return handleGetUserDictionary();
		case 'setUserDictionary':
			return handleSetUserDictionary(message);
		case 'getActivationKey':
			return handleGetActivationKey();
		case 'setActivationKey':
			return handleSetActivationKey(message);
		case 'getHotkey':
			return handleGetHotkey();
		case 'setHotkey':
			return handleSetHotkey(message);
		case 'openReportError':
			return handleOpenReportError(message);
		case 'openOptions':
			chrome.runtime.openOptionsPage();
			return Promise.resolve(createUnitResponse());
		case 'postFormData':
			return handlePostFormData(message);
		case 'getInstalledOn':
			return handleGetInstalledOn(message);
		case 'getReviewed':
			return handleGetReviewed(message);
		case 'setReviewed':
			return handleSetReviewed(message);
		case 'getWeirpacks':
			return handleGetWeirpacks();
		case 'addWeirpack':
			return handleAddWeirpack(message);
		case 'removeWeirpack':
			return handleRemoveWeirpack(message);
	}
}

/** Handle a request for linting. */
async function handleLint(req: LintRequest): Promise<LintResponse> {
	if (!(await enabledForDomain(req.domain))) {
		return { kind: 'lints', lints: {} };
	}

	const grouped = await linter.organizedLints(req.text, req.options);
	const unpackedEntries = await Promise.all(
		Object.entries(grouped).map(async ([source, lints]) => {
			const unpacked = await Promise.all(lints.map((lint) => unpackLint(req.text, lint, linter)));
			return [source, unpacked] as const;
		}),
	);
	const unpackedBySource = Object.fromEntries(unpackedEntries) as UnpackedLintGroups;
	return { kind: 'lints', lints: unpackedBySource };
}

async function handleGetConfig(_req: GetConfigRequest): Promise<GetConfigResponse> {
	return { kind: 'getConfig', config: await getLintConfig() };
}

async function handleSetConfig(req: SetConfigRequest): Promise<UnitResponse> {
	await setLintConfig(req.config);

	return createUnitResponse();
}

async function handleSetDialect(req: SetDialectRequest): Promise<UnitResponse> {
	await setDialect(req.dialect);

	return createUnitResponse();
}

async function handleGetDialect(_req: GetDialectRequest): Promise<GetDialectResponse> {
	return { kind: 'getDialect', dialect: await getDialect() };
}

async function handleIgnoreLint(req: IgnoreLintRequest): Promise<UnitResponse> {
	await linter.ignoreLintHash(BigInt(req.contextHash));
	await setIgnoredLints(await linter.exportIgnoredLints());

	return createUnitResponse();
}

async function handleGetDefaultStatus(): Promise<GetDefaultStatusResponse> {
	return {
		kind: 'getDefaultStatus',
		enabled: await enabledByDefault(),
	};
}

async function handleGetEnabledDomains(): Promise<GetEnabledDomainsResponse> {
	const all = await chrome.storage.local.get(null as any);
	const prefix = formatDomainKey(''); // yields 'domainStatus '
	const domains = Object.entries(all)
		.filter(([k, v]) => typeof v === 'boolean' && v === true && k.startsWith(prefix))
		.map(([k]) => k.substring(prefix.length))
		.sort((a, b) => a.localeCompare(b));

	return { kind: 'getEnabledDomains', domains };
}

async function handleGetDomainStatus(
	req: GetDomainStatusRequest,
): Promise<GetDomainStatusResponse> {
	return {
		kind: 'getDomainStatus',
		domain: req.domain,
		enabled: await enabledForDomain(req.domain),
	};
}

async function handleSetDomainStatus(req: SetDomainStatusRequest): Promise<UnitResponse> {
	await setDomainEnable(req.domain, req.enabled, req.overrideValue);

	return createUnitResponse();
}

async function handleSetDefaultStatus(req: SetDefaultStatusRequest): Promise<UnitResponse> {
	await setDefaultEnable(req.enabled);

	return createUnitResponse();
}

async function handleGetLintDescriptions(
	_req: GetLintDescriptionsRequest,
): Promise<GetLintDescriptionsResponse> {
	return { kind: 'getLintDescriptions', descriptions: await linter.getLintDescriptionsHTML() };
}

async function handleSetUserDictionary(req: SetUserDictionaryRequest): Promise<UnitResponse> {
	await resetDictionary();
	await addToDictionary(req.words);

	return createUnitResponse();
}

async function handleAddToUserDictionary(req: AddToUserDictionaryRequest): Promise<UnitResponse> {
	await addToDictionary(req.words);

	return createUnitResponse();
}

async function handleGetUserDictionary(): Promise<GetUserDictionaryResponse> {
	const dict = await getUserDictionary();

	return { kind: 'getUserDictionary', words: dict };
}

async function handleGetActivationKey(): Promise<GetActivationKeyResponse> {
	const key = await getActivationKey();

	return { kind: 'getActivationKey', key };
}

async function handleSetActivationKey(req: SetActivationKeyRequest): Promise<UnitResponse> {
	if (!Object.values(ActivationKey).includes(req.key)) {
		throw new Error(`Invalid activation key: ${req.key}`);
	}
	await setActivationKey(req.key);

	return createUnitResponse();
}

async function handleGetHotkey(): Promise<GetHotkeyResponse> {
	const hotkey = await getHotkey();

	return { kind: 'getHotkey', hotkey };
}

async function handleSetHotkey(req: SetHotkeyRequest): Promise<UnitResponse> {
	// Create a plain object to avoid proxy cloning issues
	const hotkey = {
		modifiers: [...req.hotkey.modifiers],
		key: req.hotkey.key,
	};
	await setHotkey(hotkey);
}

async function handleOpenReportError(req: OpenReportErrorRequest): Promise<UnitResponse> {
	const popupState: PopupState = {
		page: 'report-error',
		example: req.example,
		rule_id: req.rule_id,
		feedback: req.feedback,
	};

	await chrome.storage.local.set({ popupState });

	if (chrome.action?.openPopup) {
		try {
			await chrome.action.openPopup();
		} catch (error) {
			console.error('Failed to open popup for report error', error);
		}
	}

	return createUnitResponse();
}

async function handlePostFormData(req: PostFormDataRequest): Promise<PostFormDataResponse> {
	const formData = new FormData();
	for (const [key, value] of Object.entries(req.formData)) {
		formData.append(key, value);
	}

	try {
		const response = await fetch(req.url, {
			method: 'POST',
			body: formData,
		});

		return { kind: 'postFormData', success: response.ok };
	} catch (error) {
		console.error('Failed to post form data', error);
		return { kind: 'postFormData', success: false };
	}
}

async function handleGetInstalledOn(_req: GetInstalledOnRequest): Promise<GetInstalledOnResponse> {
	return { kind: 'getInstalledOn', installedOn: await getInstalledOn() };
}

async function handleGetReviewed(_req: GetReviewedRequest): Promise<GetReviewedResponse> {
	return { kind: 'getReviewed', reviewed: await getReviewed() };
}

async function handleSetReviewed(req: SetReviewedRequest): Promise<UnitResponse> {
	await setReviewed(req.reviewed);
	return createUnitResponse();
}

async function handleGetWeirpacks(): Promise<GetWeirpacksResponse> {
	const stored = await getStoredWeirpacks();
	const weirpacks: WeirpackMeta[] = stored.map((item) => ({
		id: item.id,
		name: item.name,
		filename: item.filename,
		version: item.version,
		installedAt: item.installedAt,
	}));

	return { kind: 'getWeirpacks', weirpacks };
}

async function handleAddWeirpack(req: AddWeirpackRequest): Promise<UnitResponse> {
	const bytes = Uint8Array.from(req.bytes);
	const failures = await linter.loadWeirpackFromBytes(bytes);
	if (failures !== undefined) {
		throw new Error(
			`This Weirpack has failing tests (${Object.keys(failures).length} rule(s) failed) and was not loaded.`,
		);
	}

	const manifest = unpackWeirpackBytes(bytes).manifest;
	const candidateName = manifest.name;
	const candidateVersion = manifest.version;
	const name =
		typeof candidateName === 'string' && candidateName.trim().length > 0
			? candidateName.trim()
			: req.filename;
	const version =
		typeof candidateVersion === 'string' && candidateVersion.trim().length > 0
			? candidateVersion.trim()
			: null;

	const current = await getStoredWeirpacks();
	current.push({
		id: createWeirpackId(),
		name,
		filename: req.filename,
		version,
		installedAt: new Date().toISOString(),
		bytesBase64: bytesToBase64(bytes),
	});
	await setStoredWeirpacks(current);

	return createUnitResponse();
}

async function handleRemoveWeirpack(req: RemoveWeirpackRequest): Promise<UnitResponse> {
	const current = await getStoredWeirpacks();
	const next = current.filter((item) => item.id !== req.id);
	await setStoredWeirpacks(next);

	initializeLinter(await linter.getDialect());
	return createUnitResponse();
}

/** Set the lint configuration inside the global `linter` and in permanent storage. */
async function setLintConfig(lintConfig: LintConfig): Promise<void> {
	await linter.setLintConfig(lintConfig);

	const json = await linter.getLintConfigAsJSON();

	await chrome.storage.local.set({ lintConfig: json });
}

/** Get the lint configuration from permanent storage. */
async function getLintConfig(): Promise<LintConfig> {
	const json = await linter.getLintConfigAsJSON();
	const resp = await chrome.storage.local.get({ lintConfig: json });
	return JSON.parse(resp.lintConfig);
}

/** Get the ignored lint state from permanent storage. */
async function setIgnoredLints(state: string): Promise<void> {
	await linter.importIgnoredLints(state);

	const json = await linter.exportIgnoredLints();

	await chrome.storage.local.set({ ignoredLints: json });
}

/** Get the ignored lint state from permanent storage. */
async function getIgnoredLints(): Promise<string> {
	const state = await linter.exportIgnoredLints();
	const resp = await chrome.storage.local.get({ ignoredLints: state });
	return resp.ignoredLints;
}

async function getDialect(): Promise<Dialect> {
	const resp = await chrome.storage.local.get('dialect');

	// If user hasn't set a dialect, try to detect from browser language
	if (resp.dialect === undefined) {
		return detectBrowserDialect();
	}

	return resp.dialect;
}

async function getActivationKey(): Promise<ActivationKey> {
	const resp = await chrome.storage.local.get({ activationKey: ActivationKey.Off });
	return resp.activationKey;
}

async function getHotkey(): Promise<Hotkey> {
	const resp = await chrome.storage.local.get({ hotkey: { modifiers: ['Ctrl'], key: 'e' } });
	return resp.hotkey;
}

async function setActivationKey(key: ActivationKey) {
	await chrome.storage.local.set({ activationKey: key });
}

async function setHotkey(hotkey: Hotkey) {
	await chrome.storage.local.set({ hotkey: hotkey });
}

function initializeLinter(dialect: Dialect) {
	if (linter != null) {
		linter.dispose();
	}

	linter = new LocalLinter({
		binary: BinaryModule.create(chrome.runtime.getURL('./wasm/harper_wasm_bg.wasm')),
		dialect,
	});

	getIgnoredLints().then((i) => linter.importIgnoredLints(i));
	getUserDictionary().then((u) => linter.importWords(u));
	getLintConfig().then((c) => linter.setLintConfig(c));
	loadStoredWeirpacksIntoLinter();
	linter.setup();
}

async function setDialect(dialect: Dialect) {
	await chrome.storage.local.set({ dialect });
	initializeLinter(dialect);
}

/** Format the key to be used in local storage to store domain status. */
function formatDomainKey(domain: string): string {
	return `domainStatus ${domain}`;
}

/** Check if Harper has been enabled for a given domain. */
async function enabledForDomain(domain: string): Promise<boolean | null> {
	const req = await chrome.storage.local.get({
		[formatDomainKey(domain)]: await enabledByDefault(),
	});
	return req[formatDomainKey(domain)];
}

/** Set whether Harper is enabled for a given domain.
 *
 * @param overrideValue dictates whether this should override a previous setting.
 * */
async function setDomainEnable(domain: string, status: boolean, overrideValue = true) {
	let shouldSet = !(await isDomainSet(domain));

	if (overrideValue) {
		shouldSet = true;
	}

	if (shouldSet) {
		await chrome.storage.local.set({ [formatDomainKey(domain)]: status });
	}
}

/** Set whether Harper is enabled by default. */
async function setDefaultEnable(status: boolean) {
	await chrome.storage.local.set({ defaultEnable: status });
}

/** Check if Harper has been enabled by default. */
async function enabledByDefault(): Promise<boolean> {
	const req = await chrome.storage.local.get({ defaultEnable: false });
	return req.defaultEnable;
}

/** Check whether Harper's state has been set for a given domain. */
async function isDomainSet(domain: string): Promise<boolean> {
	const resp = await chrome.storage.local.get(formatDomainKey(domain));
	return typeof resp[formatDomainKey(domain)] == 'boolean';
}

/** Reset the persistent user dictionary. */
async function resetDictionary(): Promise<void> {
	await chrome.storage.local.set({ userDictionary: null });

	initializeLinter(await linter.getDialect());
}

/** Add words to the persistent user dictionary. */
async function addToDictionary(words: string[]): Promise<void> {
	const exported = await linter.exportWords();
	exported.push(...words);

	await Promise.all([
		linter.importWords(exported),
		chrome.storage.local.set({ userDictionary: exported }),
	]);
}

/** Grab the user dictionary from persistent storage. */
async function getUserDictionary(): Promise<string[]> {
	const resp = await chrome.storage.local.get({ userDictionary: [] });
	return resp.userDictionary;
}

/** Record the date the extension was installed, if it's missing. */
async function setInstalledOnIfMissing(): Promise<void> {
	const current = await getInstalledOn();
	if (current !== null) {
		return;
	}

	const installedOn = new Date().toISOString();
	await chrome.storage.local.set({ installedOn });
}

async function getInstalledOn(): Promise<string | null> {
	const resp = await chrome.storage.local.get({ installedOn: null });
	return resp.installedOn;
}

async function getReviewed(): Promise<boolean> {
	const resp = await chrome.storage.local.get({ reviewed: false });
	return Boolean(resp.reviewed);
}

async function setReviewed(reviewed: boolean): Promise<void> {
	await chrome.storage.local.set({ reviewed });
}

type StoredWeirpack = WeirpackMeta & {
	bytesBase64: string;
};

function createWeirpackId(): string {
	return crypto.randomUUID?.() ?? `${Date.now()}-${Math.random()}`;
}

function bytesToBase64(bytes: Uint8Array): string {
	let binary = '';
	for (const byte of bytes) {
		binary += String.fromCharCode(byte);
	}
	return btoa(binary);
}

function base64ToBytes(encoded: string): Uint8Array {
	const binary = atob(encoded);
	const bytes = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i++) {
		bytes[i] = binary.charCodeAt(i);
	}
	return bytes;
}

async function setStoredWeirpacks(weirpacks: StoredWeirpack[]): Promise<void> {
	await chrome.storage.local.set({ [WEIRPACKS_KEY]: weirpacks });
}

async function getStoredWeirpacks(): Promise<StoredWeirpack[]> {
	const response = await chrome.storage.local.get({ [WEIRPACKS_KEY]: [] as StoredWeirpack[] });
	const value = response[WEIRPACKS_KEY];
	return Array.isArray(value) ? value : [];
}

async function loadStoredWeirpacksIntoLinter(): Promise<void> {
	const weirpacks = await getStoredWeirpacks();
	for (const weirpack of weirpacks) {
		try {
			const failures = await linter.loadWeirpackFromBytes(base64ToBytes(weirpack.bytesBase64));
			if (failures !== undefined) {
				console.error(`Stored Weirpack ${weirpack.name} failed tests`, failures);
			}
		} catch (error) {
			console.error(`Failed to load stored Weirpack ${weirpack.name}`, error);
		}
	}
}
