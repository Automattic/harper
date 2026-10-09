export const SCHEMA_VERSION = 1;

export const MAX_FILE_BYTES = 1_000_000;
export const MAX_DICTIONARY_WORDS = 50_000;
export const MAX_WORD_LENGTH = 128;
export const MAX_RULES = 5_000;
export const MAX_DOMAINS = 10_000;
export const MAX_DELAY = 60_000;
export const MAX_DOMAIN_LENGTH = 253;

// Keeping it consistent with
export const DIALECT_NAMES = ['American', 'British', 'Australian', 'Canadian', 'Indian'] as const;
export type DialectName = (typeof DIALECT_NAMES)[number];

export const ACTIVATION_KEYS = ['off', 'shift', 'control'] as const;
export type ActivationKeyName = (typeof ACTIVATION_KEYS)[number];

export const HOTKEY_MODIFIERS = ['Ctrl', 'Shift', 'Alt'] as const;
export type HotkeyModifier = (typeof HOTKEY_MODIFIERS)[number];

// Settings understood by every harper integration
export type CoreSettings = {
	dialect: DialectName;
	lint_config: Record<string, boolean | null>; // null means follow harper's default for this rule
	user_dictionary: string[];
	ignored_lints: string; // linter's own ignoredlint export
	// contains 64bit hashes that `JSON.parse` would silently round
};

// Settings specific to browser extension
export type ExtensionSettings = {
	domain_status: Record<string, boolean>;
	default_enabled: boolean;
	delay: number;
	activation_key: ActivationKeyName;
	hotkey: { modifiers: HotkeyModifier[]; key: string };
	isolate_english: boolean;
};

export type SettingsFile = {
	schema_version: number;
	exported_at: string;
	source_app: string;
	harper_version: string;
	core: CoreSettings;
	extension?: ExtensionSettings;
};

export class SettingsParseError extends Error {
	constructor(message: string) {
		super(message);
		this.name = 'SettingsParseError';
	}
}

const MAX_U64 = 2n ** 64n - 1n;
const IGNORED_LINTS_PATTERN =
	/^\s*\{\s*"context_hashes"\s*:\s*\[\s*(\d{1,20}\s*(,\s*\d{1,20}\s*)*)?\]\s*\}\s*$/;
const DOMAIN_PATTERN = /^[A-Za-z0-9._\-[\]:]+$/;
// biome-ignore lint/suspicious/noControlCharactersInRegex: rejecting control characters is the point.
const CONTROL_CHARS = /[\u0000-\u001f\u007f]/;

/**
 * Parse and validate the contents of a settings file.
 * Throws a `SettingsParseError` without side effects if anything is wrong.
 * Unknown toplevel blocks and fields are dropped.
 */
export function parseSettings(raw: string): SettingsFile {
	if (typeof raw !== 'string') {
		throw new SettingsParseError('Settings file must be text.');
	}
	if (new TextEncoder().encode(raw).length > MAX_FILE_BYTES) {
		throw new SettingsParseError(`Settings file is larger than ${MAX_FILE_BYTES} bytes.`);
	}
	let data: unknown;
	try {
		data = JSON.parse(raw);
	} catch {
		throw new SettingsParseError('Settings file is not valid JSON.');
	}

	const root = expectObject(data, 'root');
	const schemaVersion = root.schema_version;
	if (typeof schemaVersion !== 'number' || !Number.isInteger(schemaVersion) || schemaVersion < 1) {
		throw new SettingsParseError('Missing or invalid schema_version.');
	}
	if (schemaVersion > SCHEMA_VERSION) {
		throw new SettingsParseError(
			`This file uses schema_version ${schemaVersion}, but this version of Harper only supports up to ${SCHEMA_VERSION}. Update Harper and try again`,
		);
	}
	const settings: SettingsFile = {
		schema_version: schemaVersion,
		exported_at: optionalString(root.exported_at, 'exported_at'),
		source_app: optionalString(root.source_app, 'source_app'),
		harper_version: optionalString(root.harper_version, 'harper_version'),
		core: parseCore(root.core),
	};

	if (root.extension !== undefined) {
		settings.extension = parseExtension(root.extension);
	}
	return settings;
}

function parseCore(value: unknown): CoreSettings {
	const core = expectObject(value, 'core');

	if (!isDialectName(core.dialect)) {
		throw new SettingsParseError(
			`Invalid core.dialect. Expected one of ${DIALECT_NAMES.join(',')}.`,
		);
	}
	return {
		dialect: core.dialect,
		lint_config: parseLintConfig(core.lint_config),
		user_dictionary: parseDictionary(core.user_dictionary),
		ignored_lints: parseIgnoredLints(core.ignored_lints),
	};
}

function parseLintConfig(value: unknown): Record<string, boolean | null> {
	const config = expectObject(value, 'core.lint_config');
	const entries = Object.entries(config);

	if (entries.length > MAX_RULES) {
		throw new SettingsParseError('core.lint_config has too many rules.');
	}

	for (const [name, setting] of entries) {
		if (name.length === 0 || name.length > MAX_WORD_LENGTH) {
			throw new SettingsParseError(`Invalid rule name in core.lint_config: "${name}".`);
		}
		if (setting !== null && typeof setting !== 'boolean') {
			throw new SettingsParseError(`Rule "${name}" must be true, false, or null.`);
		}
	}

	return Object.fromEntries(entries) as Record<string, boolean | null>;
}

function parseDictionary(value: unknown): string[] {
	if (!Array.isArray(value)) {
		throw new SettingsParseError('core.user_dictionary must be a list of words.');
	}
	if (value.length > MAX_DICTIONARY_WORDS) {
		throw new SettingsParseError(
			`core.user_dictionary has more than ${MAX_DICTIONARY_WORDS} words.`,
		);
	}

	for (const word of value) {
		if (
			typeof word !== 'string' ||
			word.trim().length === 0 ||
			word.length > MAX_WORD_LENGTH ||
			CONTROL_CHARS.test(word)
		) {
			throw new SettingsParseError(
				`Invalid word in core.user_dictionary: ${JSON.stringify(word)}.`,
			);
		}
	}

	return [...value];
}

function parseIgnoredLints(value: unknown): string {
	if (
		typeof value !== 'string' ||
		!IGNORED_LINTS_PATTERN.test(value) ||
		(value.match(/\d+/g) ?? []).some((hash) => BigInt(hash) > MAX_U64)
	) {
		throw new SettingsParseError('Invalid core.ignored_lints.');
	}

	return value;
}

function parseExtension(value: unknown): ExtensionSettings {
	const ext = expectObject(value, 'extension');

	if (typeof ext.default_enabled !== 'boolean') {
		throw new SettingsParseError('extension.default_enabled must be true or false.');
	}
	if (
		typeof ext.delay !== 'number' ||
		!Number.isFinite(ext.delay) ||
		ext.delay < 0 ||
		ext.delay > MAX_DELAY
	) {
		throw new SettingsParseError(`extension.delay must be a number between 0 and ${MAX_DELAY}.`);
	}
	if (!(ACTIVATION_KEYS as readonly unknown[]).includes(ext.activation_key)) {
		throw new SettingsParseError(
			`Invalid extension.activation_key. Expected one of ${ACTIVATION_KEYS.join(', ')}.`,
		);
	}
	if (typeof ext.isolate_english !== 'boolean') {
		throw new SettingsParseError('extension.isolate_english must be true or false.');
	}

	return {
		domain_status: parseDomainStatus(ext.domain_status),
		default_enabled: ext.default_enabled,
		delay: Math.trunc(ext.delay),
		activation_key: ext.activation_key as ActivationKeyName,
		hotkey: parseHotkey(ext.hotkey),
		isolate_english: ext.isolate_english,
	};
}

function parseDomainStatus(value: unknown): Record<string, boolean> {
	const status = expectObject(value, 'extension.domain_status');
	const entries = Object.entries(status);

	if (entries.length > MAX_DOMAINS) {
		throw new SettingsParseError('extension.domain_status has too many domains.');
	}

	for (const [domain, enabled] of entries) {
		if (domain.length > MAX_DOMAIN_LENGTH || !DOMAIN_PATTERN.test(domain)) {
			throw new SettingsParseError(`Invalid domain in extension.domain_status: "${domain}".`);
		}
		if (typeof enabled !== 'boolean') {
			throw new SettingsParseError(`Domain "${domain}" must be true or false.`);
		}
	}

	return Object.fromEntries(entries) as Record<string, boolean>;
}

function parseHotkey(value: unknown): ExtensionSettings['hotkey'] {
	const hotkey = expectObject(value, 'extension.hotkey');
	const mods = hotkey.modifiers;

	if (
		!Array.isArray(mods) ||
		!mods.every((m) => (HOTKEY_MODIFIERS as readonly unknown[]).includes(m))
	) {
		throw new SettingsParseError(
			`extension.hotkey.modifiers must only contain ${HOTKEY_MODIFIERS.join(', ')}.`,
		);
	}
	if (new Set(mods).size !== mods.length) {
		throw new SettingsParseError('extension.hotkey.modifiers must not repeat.');
	}
	if (
		typeof hotkey.key !== 'string' ||
		hotkey.key.length === 0 ||
		hotkey.key.length > 32 ||
		CONTROL_CHARS.test(hotkey.key)
	) {
		throw new SettingsParseError('Invalid extension.hotkey.key.');
	}

	return { modifiers: [...mods] as HotkeyModifier[], key: hotkey.key };
}

function expectObject(value: unknown, path: string): Record<string, unknown> {
	if (typeof value !== 'object' || value === null || Array.isArray(value)) {
		throw new SettingsParseError(`Missing or invalid ${path}.`);
	}

	return value as Record<string, unknown>;
}

function optionalString(value: unknown, path: string): string {
	if (value === undefined) {
		return '';
	}
	if (typeof value !== 'string' || value.length > 256) {
		throw new SettingsParseError(`Invalid ${path}.`);
	}

	return value;
}

export function isDialectName(value: unknown): value is DialectName {
	return (DIALECT_NAMES as readonly unknown[]).includes(value);
}
