import type { Dialect, LintConfig } from 'harper.js';
import { LRUCache } from 'lru-cache';
import type { UnpackedLint } from './unpackLint';

export default class ProtocolClient {
	private static readonly lintCache = new LRUCache<string, UnpackedLint[]>({
		max: 500,
		ttl: 5_000,
	});

	private static cacheKey(text: string, domain: string): string {
		return `${domain}:${text}`;
	}

	public static async lint(text: string, domain: string): Promise<UnpackedLint[]> {
		const key = this.cacheKey(text, domain);
		const cached = this.lintCache.get(key);
		if (cached) return cached;
		const resp = await chrome.runtime.sendMessage({ kind: 'lint', text, domain });
		this.lintCache.set(key, resp.lints);
		return resp.lints;
	}

	public static async getLintConfig(): Promise<LintConfig> {
		return (await chrome.runtime.sendMessage({ kind: 'getConfig' })).config;
	}

	public static async setLintConfig(lintConfig: LintConfig): Promise<void> {
		this.lintCache.clear();
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
		return (await chrome.runtime.sendMessage({ kind: 'getDomainStatus', domain })).enabled;
	}

	public static async setDomainEnabled(domain: string, enabled: boolean): Promise<void> {
		await chrome.runtime.sendMessage({ kind: 'setDomainStatus', enabled, domain });
	}

	public static async addToUserDictionary(word: string): Promise<void> {
		this.lintCache.clear();
		await chrome.runtime.sendMessage({ kind: 'addToUserDictionary', word });
	}

	public static async ignoreHash(hash: string): Promise<void> {
		await chrome.runtime.sendMessage({ kind: 'ignoreLint', contextHash: hash });
		this.lintCache.clear();
	}
}
