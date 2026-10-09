import type { CoreSettings, DialectName, ExtensionSettings, SettingsFile } from './schema';
import { SCHEMA_VERSION } from './schema';

export type ImportMode = 'merge' | 'replace';

export type ImportOptions = {
	mode: ImportMode;
	/** Whether to apply the `extension` block (domains, delay, hotkey, ...). */
	includeExtension: boolean;
};

/** The current state of this browser profile, in file form. */
export type CurrentSettings = {
	core: CoreSettings;
	extension: ExtensionSettings;
};

/**
 * The storage changes required to apply an import.
 * `dialect` is kept separate because storage holds the `harper.js` enum value, not its name.
 */
export type ImportPlan = {
	dialect: DialectName;
	set: Record<string, unknown>;
	remove: string[];
};

export type ImportSummary = {
	dialectChanged: boolean;
	wordsAdded: number;
	wordsRemoved: number;
	ruleChanges: number;
	ignoredLintsAdded: number;
	domainChanges: number;
	extensionIncluded: boolean;
};

export const DOMAIN_STATUS_PREFIX = 'domainStatus ';

export function formatDomainKey(domain: string): string {
	return `${DOMAIN_STATUS_PREFIX}${domain}`;
}

/** Build the file contents for the current state. */
export function buildExport(
	current: CurrentSettings,
	meta: { sourceApp: string; harperVersion: string; now?: Date },
): SettingsFile {
	return {
		schema_version: SCHEMA_VERSION,
		exported_at: (meta.now ?? new Date()).toISOString(),
		source_app: meta.sourceApp,
		harper_version: meta.harperVersion,
		core: {
			...current.core,
			user_dictionary: dedupWords(current.core.user_dictionary),
			ignored_lints: formatIgnoredLints(extractIgnoredLintHashes(current.core.ignored_lints)),
		},
		extension: current.extension,
	};
}

/** Work out what the settings should be after importing `incoming`. */
export function computeImportedSettings(
	current: CurrentSettings,
	incoming: SettingsFile,
	{ mode, includeExtension }: ImportOptions,
): CurrentSettings {
	const merge = mode === 'merge';
	const cur = current.core;
	const inc = incoming.core;

	const core: CoreSettings = {
		dialect: inc.dialect,
		lint_config: merge ? { ...cur.lint_config, ...inc.lint_config } : { ...inc.lint_config },
		user_dictionary: dedupWords(
			merge ? [...cur.user_dictionary, ...inc.user_dictionary] : inc.user_dictionary,
		),
		ignored_lints: formatIgnoredLints(
			merge
				? [
						...extractIgnoredLintHashes(cur.ignored_lints),
						...extractIgnoredLintHashes(inc.ignored_lints),
					]
				: extractIgnoredLintHashes(inc.ignored_lints),
		),
	};

	if (!includeExtension || incoming.extension === undefined) {
		return { core, extension: current.extension };
	}

	const ext = incoming.extension;
	return {
		core,
		extension: {
			...ext,
			domain_status: merge
				? { ...current.extension.domain_status, ...ext.domain_status }
				: { ...ext.domain_status },
			hotkey: { modifiers: [...ext.hotkey.modifiers], key: ext.hotkey.key },
		},
	};
}

/** Turn an import into concrete `chrome.storage.local` changes. */
export function planImport(
	current: CurrentSettings,
	incoming: SettingsFile,
	options: ImportOptions,
): ImportPlan {
	const next = computeImportedSettings(current, incoming, options);
	const set: Record<string, unknown> = {
		lintConfig: JSON.stringify(next.core.lint_config),
		userDictionary: next.core.user_dictionary,
		ignoredLints: next.core.ignored_lints,
	};
	const remove: string[] = [];

	if (options.includeExtension && incoming.extension !== undefined) {
		const ext = next.extension;
		set.defaultEnable = ext.default_enabled;
		set.delay = ext.delay;
		set.activationKey = ext.activation_key;
		set.hotkey = ext.hotkey;
		set.isolateEnglish = ext.isolate_english;

		for (const [domain, enabled] of Object.entries(ext.domain_status)) {
			set[formatDomainKey(domain)] = enabled;
		}
		for (const domain of Object.keys(current.extension.domain_status)) {
			if (!(domain in ext.domain_status)) {
				remove.push(formatDomainKey(domain));
			}
		}
	}

	return { dialect: next.core.dialect, set, remove };
}

/** Counts shown to the user before they confirm an import. */
export function summarizeImport(
	current: CurrentSettings,
	incoming: SettingsFile,
	options: ImportOptions,
): ImportSummary {
	const next = computeImportedSettings(current, incoming, options);

	const curWords = new Set(current.core.user_dictionary);
	const nextWords = new Set(next.core.user_dictionary);
	const curHashes = new Set(extractIgnoredLintHashes(current.core.ignored_lints));
	const nextHashes = extractIgnoredLintHashes(next.core.ignored_lints);

	return {
		dialectChanged: current.core.dialect !== next.core.dialect,
		wordsAdded: [...nextWords].filter((w) => !curWords.has(w)).length,
		wordsRemoved: [...curWords].filter((w) => !nextWords.has(w)).length,
		ruleChanges: countRecordChanges(current.core.lint_config, next.core.lint_config, null),
		ignoredLintsAdded: nextHashes.filter((h) => !curHashes.has(h)).length,
		domainChanges: countRecordChanges(
			current.extension.domain_status,
			next.extension.domain_status,
			undefined,
		),
		extensionIncluded: options.includeExtension && incoming.extension !== undefined,
	};
}

/** Remove empty and duplicate words, keeping the first occurrence's order. */
export function dedupWords(words: string[]): string[] {
	return [...new Set(words.map((w) => w.trim()).filter((w) => w.length > 0))];
}

/**
 * Pull the hashes out of the linter's ignored-lint JSON as strings.
 * Parsing them as numbers would lose precision above 2^53.
 */
export function extractIgnoredLintHashes(json: string): string[] {
	const list = /"context_hashes"\s*:\s*\[([^\]]*)\]/.exec(json)?.[1] ?? '';
	return list.match(/\d+/g)?.map(stripLeadingZeros) ?? [];
}

export function formatIgnoredLints(hashes: string[]): string {
	return `{"context_hashes":[${[...new Set(hashes)].join(',')}]}`;
}

function stripLeadingZeros(digits: string): string {
	return digits.replace(/^0+(?=\d)/, '');
}

function countRecordChanges<T>(
	before: Record<string, T>,
	after: Record<string, T>,
	missing: T | undefined,
): number {
	const keys = new Set([...Object.keys(before), ...Object.keys(after)]);
	return [...keys].filter((k) => (before[k] ?? missing) !== (after[k] ?? missing)).length;
}
