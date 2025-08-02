import { BinaryModule, Dialect, type LintConfig, LocalLinter } from 'harper.js';
import {
	type AddToUserDictionaryRequest,
	createUnitResponse,
	type GetConfigRequest,
	type GetConfigResponse,
	type GetDefaultStatusResponse,
	type GetDialectRequest,
	type GetDialectResponse,
	type GetDomainStatusRequest,
	type GetDomainStatusResponse,
	type GetLintDescriptionsRequest,
	type GetLintDescriptionsResponse,
	type GetUserDictionaryResponse,
	type IgnoreLintRequest,
	type LintRequest,
	type LintResponse,
	type Request,
	type Response,
	type SetConfigRequest,
	type SetDefaultStatusRequest,
	type SetDialectRequest,
	type SetDomainStatusRequest,
	type SetUserDictionaryRequest,
	type UnitResponse,
} from '../protocol';
import unpackLint from '../unpackLint';

console.log('background is running');

chrome.runtime.onInstalled.addListener((details) => {
	if (details.reason === chrome.runtime.OnInstalledReason.INSTALL) {
		chrome.runtime.setUninstallURL('https://writewithharper.com/uninstall-browser-extension');
		chrome.tabs.create({ url: 'https://writewithharper.com/install-browser-extension' });
	}
});

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
	handleRequest(request).then(sendResponse);

	return true;
});

let linter: LocalLinter;

getDialect().then(setDialect);

async function enableDefaultDomains() {
	const defaultEnabledDomains = [
		'old.reddit.com',
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
		'www.google.com',
		'www.instagram.com',
		'web.whatsapp.com',
		'outlook.live.com',
		'www.reddit.com',
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
		case 'getUserDictionary':
			return handleGetUserDictionary();
		case 'setUserDictionary':
			return handleSetUserDictionary(message);
	}
}

/** Handle a request for linting. */
async function handleLint(req: LintRequest): Promise<LintResponse> {
	if (!(await enabledForDomain(req.domain))) {
		return { kind: 'lints', lints: [] };
	}

	const lints = await linter.lint(req.text);
	const unpackedLints = await Promise.all(lints.map((l) => unpackLint(req.text, l, linter)));
	return { kind: 'lints', lints: unpackedLints };
}

async function handleGetConfig(req: GetConfigRequest): Promise<GetConfigResponse> {
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

async function handleGetDialect(req: GetDialectRequest): Promise<GetDialectResponse> {
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
	await setDomainEnable(req.domain, req.enabled);

	return createUnitResponse();
}

async function handleSetDefaultStatus(req: SetDefaultStatusRequest): Promise<UnitResponse> {
	await setDefaultEnable(req.enabled);

	return createUnitResponse();
}

async function handleGetLintDescriptions(
	req: GetLintDescriptionsRequest,
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
	const resp = await chrome.storage.local.get({ dialect: Dialect.American });
	return resp.dialect;
}

function initializeLinter(dialect: Dialect) {
	linter = new LocalLinter({
		binary: new BinaryModule(chrome.runtime.getURL('./wasm/harper_wasm_bg.wasm')),
		dialect,
	});

	getIgnoredLints().then((i) => linter.importIgnoredLints(i));
	getUserDictionary().then((u) => linter.importWords(u));
	getLintConfig().then((c) => linter.setLintConfig(c));
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
async function enabledForDomain(domain: string): Promise<boolean> {
	const req = await chrome.storage.local.get({
		[formatDomainKey(domain)]: await enabledByDefault(),
	});
	return req[formatDomainKey(domain)];
}

/** Set whether Harper is enabled for a given domain. */
async function setDomainEnable(domain: string, status: boolean) {
	await chrome.storage.local.set({ [formatDomainKey(domain)]: status });
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
