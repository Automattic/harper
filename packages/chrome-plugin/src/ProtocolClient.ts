import type { Dialect, LintConfig } from 'harper.js';
import { LRUCache } from 'lru-cache';
import type { UnpackedLint } from './unpackLint';

/** A wrapper around Chrome's messaging protocol for communicating with the background worker. */
export default class ProtocolClient {
	public static async lint(text: string, domain: string): Promise<UnpackedLint[]> {
		return (await chrome.runtime.sendMessage({ kind: 'lint', text, domain })).lints;
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

	public static async getDomainEnabled(domain: string): Promise<boolean> {
		const resp = await chrome.runtime.sendMessage({ kind: 'getDomainStatus', domain });

		return resp.enabled;
	}

	public static async setDomainEnabled(domain: string, enabled: boolean): Promise<void> {
		await chrome.runtime.sendMessage({ kind: 'setDomainStatus', enabled, domain });
	}

	public static async addToUserDictionary(word: string): Promise<void> {
		await chrome.runtime.sendMessage({ kind: 'addToUserDictionary', word });
	}
}
