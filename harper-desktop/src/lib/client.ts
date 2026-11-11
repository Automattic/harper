import { invoke } from '@tauri-apps/api/core';
import { disable, enable, isEnabled } from '@tauri-apps/plugin-autostart';
import type {
	Dialect,
	Lint,
	LintConfig,
	Linter,
	LocalLinter as LocalLinterType,
	StructuredLintConfig,
} from 'harper.js';

type RustDialect = 'American' | 'British' | 'Australian' | 'Canadian' | 'Indian';

const DialectValue = {
	American: 0,
	British: 1,
	Australian: 2,
	Canadian: 3,
	Indian: 4,
} as const;

const ACCESSIBILITY_PERMISSION_TIMEOUT_MS = 6_000;

let configLinterPromise: Promise<LocalLinterType> | null = null;

function getConfigLinter(): Promise<LocalLinterType> {
	configLinterPromise ??= Promise.all([
		import('harper.js'),
		import('harper.js/slimBinaryInlined'),
	]).then(
		([{ LocalLinter }, { slimBinaryInlined }]) => new LocalLinter({ binary: slimBinaryInlined }),
	);

	return configLinterPromise;
}

/** Persisted pack metadata and original archive, shared with Rust and the highlighter. */
export interface InstalledWeirpack {
	id: string;
	name: string;
	filename: string;
	version: string;
	size: number;
	ruleCount: number;
	addedAt: number;
	enabled: boolean;
	bytesBase64: string;
}

export interface Integration {
	bundle_id: string;
	enabled: boolean;
	display_name: string;
}

export interface AppSearchResult {
	name: string;
	bundle_id: string;
}

export type AccessibilityPermissionStatus = 'Granted' | 'NotGranted' | 'Unsupported';

/** Serializable Rust updater outcome; an installed update may still require a restart. */
export interface UpdateResult {
	status: 'up-to-date' | 'updated' | 'error';
	currentVersion: string | null;
	latestVersion: string | null;
	message: string;
	error: string | null;
}

/** Provides an easy-to-use interface for interacting with the main Rust Tauri process.
 * Relevant because that is where the canonical state is stored. */
export class Client {
	/** Rebuild the editor engine from canonical settings. A fresh engine removes disabled pack rules and words. */
	static async createEditorLinter(): Promise<Linter> {
		const [{ WorkerLinter }, { slimBinaryInlined }, packs, dialect, words, config] =
			await Promise.all([
				import('harper.js'),
				import('harper.js/slimBinaryInlined'),
				Client.getWeirpacks(),
				Client.getDialect(),
				Client.getDictionary(),
				Client.getLintConfig(),
			]);
		const linter = new WorkerLinter({ binary: slimBinaryInlined, dialect });
		try {
			await linter.setup();
			await linter.importWords(words);
			for (const pack of packs.filter((pack) => pack.enabled)) {
				const bytes = Uint8Array.from(atob(pack.bytesBase64), (character) =>
					character.charCodeAt(0),
				);
				const failures = await linter.loadWeirpackFromBytes(bytes);
				if (failures && Object.keys(failures).length)
					throw new Error(`Tests failed in ${pack.name}.`);
			}
			await linter.setLintConfig({
				...(await linter.getLintConfig()),
				...Object.fromEntries(Object.entries(config).filter(([, value]) => value != null)),
			});
			return linter;
		} catch (error) {
			await linter.dispose();
			throw error;
		}
	}

	static async getWeirpacks(): Promise<InstalledWeirpack[]> {
		return invoke('get_weirpacks');
	}

	/** Rust validates rules, tests, and dictionaries before saving and notifying open editors. */
	static async installWeirpack(filename: string, bytes: Uint8Array): Promise<void> {
		await invoke('install_weirpack', { filename, bytes: Array.from(bytes) });
	}

	static async removeWeirpack(id: string): Promise<void> {
		await invoke('remove_weirpack', { id });
	}

	static async updateWeirpack(
		id: string,
		patch: { name?: string; enabled?: boolean },
	): Promise<void> {
		await invoke('update_weirpack', { id, ...patch });
	}

	static async getLintConfig(): Promise<LintConfig> {
		return await invoke<LintConfig>('get_lint_config');
	}

	static async getDefaultLintConfig(): Promise<LintConfig> {
		const configLinter = await getConfigLinter();

		return JSON.parse(await configLinter.getDefaultLintConfigAsJSON()) as LintConfig;
	}

	static async getStructuredLintConfig(): Promise<StructuredLintConfig> {
		const configLinter = await getConfigLinter();

		await configLinter.setLintConfigWithJSON(JSON.stringify(await Client.getLintConfig()));

		return JSON.parse(await configLinter.getStructuredLintConfigJSON()) as StructuredLintConfig;
	}

	static async getDialect(): Promise<Dialect> {
		return rustDialectToDialect(await invoke<RustDialect>('get_dialect'));
	}

	static async setDialect(dialect: Dialect): Promise<void> {
		await invoke('set_dialect', { dialect: dialectToRustDialect(dialect) });
	}

	static async getDebounceMs(): Promise<number> {
		return await invoke<number>('get_debounce_ms');
	}

	static async setDebounceMs(debounceMs: number): Promise<void> {
		await invoke('set_debounce_ms', { debounceMs });
	}

	/** Check if the app is configured to auto update. */
	static async getAutoUpdate(): Promise<boolean> {
		return await invoke<boolean>('get_auto_update');
	}

	/** Configure the app to auto update. */
	static async setAutoUpdate(autoUpdate: boolean): Promise<void> {
		await invoke('set_auto_update', { autoUpdate });
	}

	static async getCurrentVersion(): Promise<string> {
		return await invoke<string>('get_current_version');
	}

	static async getLatestVersion(): Promise<string> {
		return await invoke<string>('get_latest_version');
	}

	static async updateToLatest(): Promise<UpdateResult> {
		return await invoke<UpdateResult>('update_to_latest');
	}

	static async getOnboardingCompleted(): Promise<boolean> {
		return await invoke<boolean>('get_onboarding_completed');
	}

	static async setOnboardingCompleted(onboardingCompleted: boolean): Promise<void> {
		await invoke('set_onboarding_completed', { onboardingCompleted });
	}

	static async getLaunchAtStartup(): Promise<boolean> {
		return await isEnabled();
	}

	static async setLaunchAtStartup(enabled: boolean): Promise<void> {
		if (enabled) {
			await enable();
		} else {
			await disable();
		}
	}

	static async setLintConfig(lintConfig: LintConfig): Promise<void> {
		await invoke('set_lint_config', { lintConfig });
	}

	static async getDictionary(): Promise<string[]> {
		return await invoke<string[]>('get_dictionary');
	}

	static async setDictionary(words: string[]): Promise<void> {
		await invoke('set_dictionary', { words });
	}

	static async disableRule(ruleName: string): Promise<LintConfig> {
		const lintConfig = await Client.getLintConfig();
		lintConfig[ruleName] = false;

		await Client.setLintConfig(lintConfig);

		return lintConfig;
	}

	static async ignoreLint(linter: Linter, source: string, lint: Lint): Promise<void> {
		await linter.ignoreLint(source, lint);
		const ignoredLints = await linter.exportIgnoredLints();

		await invoke('ignore_lint', { ignoredLints });
	}

	static async addToDictionary(word: string): Promise<void> {
		await invoke('add_to_dictionary', { word });
	}

	static async getIntegrations(): Promise<Integration[]> {
		return await invoke<Integration[]>('get_integrations');
	}

	static async getAutoEnableNewApps(): Promise<boolean> {
		return await invoke<boolean>('get_auto_enable_new_apps');
	}

	static async setAutoEnableNewApps(autoEnableNewApps: boolean): Promise<void> {
		await invoke('set_auto_enable_new_apps', { autoEnableNewApps });
	}

	static async addIntegration(bundleId: string): Promise<void> {
		await invoke('add_integration', { bundleId });
	}

	static async removeIntegration(bundleId: string): Promise<void> {
		await invoke('remove_integration', { bundleId });
	}

	static async setIntegrationEnabled(bundleId: string, enabled: boolean): Promise<void> {
		await invoke('set_integration_enabled', { bundleId, enabled });
	}

	static async getApplicationIconDataUrl(bundleId: string): Promise<string> {
		return await invoke<string>('get_application_icon_data_url', { bundleId });
	}

	static async searchApps(query: string): Promise<AppSearchResult[]> {
		return await invoke<AppSearchResult[]>('search_apps', { query });
	}

	static async launchApp(bundleId: string): Promise<void> {
		await invoke('launch_app', { bundleId });
	}

	static async getAccessibilityPermissionStatus(): Promise<AccessibilityPermissionStatus> {
		return await withTimeout(
			invoke<AccessibilityPermissionStatus>('get_accessibility_permission_status'),
			ACCESSIBILITY_PERMISSION_TIMEOUT_MS,
			'Accessibility permission check timed out',
		);
	}

	static async requestAccessibilityPermission(): Promise<AccessibilityPermissionStatus> {
		return await withTimeout(
			invoke<AccessibilityPermissionStatus>('request_accessibility_permission'),
			ACCESSIBILITY_PERMISSION_TIMEOUT_MS,
			'Accessibility permission request timed out',
		);
	}

	static async startHighlighterService(): Promise<boolean> {
		return await invoke<boolean>('start_highlighter_service');
	}

	static async stopHighlighterService(): Promise<boolean> {
		return await invoke<boolean>('stop_highlighter_service');
	}
}

async function withTimeout<T>(promise: Promise<T>, timeoutMs: number, message: string): Promise<T> {
	let timeoutId: ReturnType<typeof setTimeout> | undefined;

	const timeout = new Promise<never>((_, reject) => {
		timeoutId = setTimeout(() => reject(new Error(message)), timeoutMs);
	});

	try {
		return await Promise.race([promise, timeout]);
	} finally {
		if (timeoutId) {
			clearTimeout(timeoutId);
		}
	}
}

function rustDialectToDialect(dialect: RustDialect): Dialect {
	switch (dialect) {
		case 'British':
			return DialectValue.British as Dialect;
		case 'Australian':
			return DialectValue.Australian as Dialect;
		case 'Canadian':
			return DialectValue.Canadian as Dialect;
		case 'Indian':
			return DialectValue.Indian as Dialect;
		default:
			return DialectValue.American as Dialect;
	}
}

function dialectToRustDialect(dialect: Dialect): RustDialect {
	switch (dialect) {
		case DialectValue.British:
			return 'British';
		case DialectValue.Australian:
			return 'Australian';
		case DialectValue.Canadian:
			return 'Canadian';
		case DialectValue.Indian:
			return 'Indian';
		default:
			return 'American';
	}
}
