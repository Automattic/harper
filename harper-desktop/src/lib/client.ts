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

export interface Integration {
	bundle_id: string;
	enabled: boolean;
}

export class Client {
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

	static async addIntegration(bundleId: string): Promise<void> {
		await invoke('add_integration', { bundleId });
	}

	static async removeIntegration(bundleId: string): Promise<void> {
		await invoke('remove_integration', { bundleId });
	}

	static async setIntegrationEnabled(bundleId: string, enabled: boolean): Promise<void> {
		await invoke('set_integration_enabled', { bundleId, enabled });
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
