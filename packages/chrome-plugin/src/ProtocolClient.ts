import type { Dialect, LintConfig } from 'harper.js';
import type { UnpackedLint } from './unpackLint';

/** A wrapper around Chrome's messaging protocol for communicating with the background worker. */
export default class ProtocolClient {
	public static async lint(text: string): Promise<UnpackedLint[]> {
		return (await chrome.runtime.sendMessage({ kind: 'lint', text })).lints;
	}

	public static async getLintConfig(): Promise<LintConfig> {
		return (await chrome.runtime.sendMessage({ kind: 'getConfig' })).config;
	}

	public static async setLintConfig(lintConfig: LintConfig): Promise<void> {
		await chrome.runtime.sendMessage({ kind: 'setConfig', config: lintConfig });
	}

	public static async getLintDescriptions(): Promise<Record<string, string>> {
		return (await chrome.runtime.sendMessage({ kind: 'getLintDescriptions' })).descriptions;
	}

	public static async getDialect(): Promise<Dialect> {
		return (await chrome.runtime.sendMessage({ kind: 'getDialect' })).dialect;
	}

	public static async setDialect(dialect: Dialect): Promise<void> {
		await chrome.runtime.sendMessage({ kind: 'setDialect', dialect });
	}
}
