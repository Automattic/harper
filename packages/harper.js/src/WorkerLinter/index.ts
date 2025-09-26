import type { Dialect, Lint, Suggestion } from 'harper-wasm';
import type { BinaryModule, DeserializedRequest } from '../binary';
import type Linter from '../Linter';
import type { LinterInit } from '../Linter';
import type { LintConfig, LintOptions } from '../main';
import Worker from './worker.ts?worker&inline';

/** The data necessary to complete a request once the worker has responded. */
export interface RequestItem {
	resolve: (item: unknown) => void;
	reject: (item: unknown) => void;
	request: DeserializedRequest;
}

/** A Linter that spins up a dedicated web worker to do processing on a separate thread.
 * Main benefit: this Linter will not block the event loop for large documents.
 *
 * NOTE: This class will not work properly in Node. In that case, just use `LocalLinter`. */
export default class WorkerLinter implements Linter {
	private binary: BinaryModule;
	private dialect?: Dialect;
	private worker: Worker;
	private requestQueue: RequestItem[];
	private working = true;

	constructor(init: LinterInit) {
		this.binary = init.binary;
		this.dialect = init.dialect;
		this.worker = new Worker();
		this.requestQueue = [];

		// Fires when the worker sends 'ready'.
		this.worker.onmessage = () => {
			this.setupMainEventListeners();

			this.worker.postMessage([this.binary.url, this.dialect]);

			this.working = false;
			this.submitRemainingRequests();
		};
	}

	private setupMainEventListeners() {
		this.worker.onmessage = (e: MessageEvent) => {
			const { resolve } = this.requestQueue.shift()!;
			this.binary.deserializeArg(e.data).then((v) => {
				resolve(v);

				this.working = false;

				this.submitRemainingRequests();
			});
		};

		this.worker.onmessageerror = (e: MessageEvent) => {
			const { reject } = this.requestQueue.shift()!;
			reject(e.data);
			this.working = false;

			this.submitRemainingRequests();
		};
	}

	setup(): Promise<void> {
		return this.rpc('setup', []);
	}

	lint(text: string, options?: LintOptions): Promise<Lint[]> {
		return this.rpc('lint', [text, options]);
	}

	applySuggestion(text: string, lint: Lint, suggestion: Suggestion): Promise<string> {
		return this.rpc('applySuggestion', [text, lint, suggestion]);
	}

	isLikelyEnglish(text: string): Promise<boolean> {
		return this.rpc('isLikelyEnglish', [text]);
	}

	isolateEnglish(text: string): Promise<string> {
		return this.rpc('isolateEnglish', [text]);
	}

	async getLintConfig(): Promise<LintConfig> {
		return JSON.parse(await this.getLintConfigAsJSON());
	}

	setLintConfig(config: LintConfig): Promise<void> {
		return this.setLintConfigWithJSON(JSON.stringify(config));
	}

	getLintConfigAsJSON(): Promise<string> {
		return this.rpc('getLintConfigAsJSON', []);
	}

	setLintConfigWithJSON(config: string): Promise<void> {
		return this.rpc('setLintConfigWithJSON', [config]);
	}

	toTitleCase(text: string): Promise<string> {
		return this.rpc('toTitleCase', [text]);
	}

	getLintDescriptionsAsJSON(): Promise<string> {
		return this.rpc('getLintDescriptionsAsJSON', []);
	}

	async getLintDescriptions(): Promise<Record<string, string>> {
		return JSON.parse(await this.getLintDescriptionsAsJSON()) as Record<string, string>;
	}

	getLintDescriptionsHTMLAsJSON(): Promise<string> {
		return this.rpc('getLintDescriptionsHTMLAsJSON', []);
	}

	async getLintDescriptionsHTML(): Promise<Record<string, string>> {
		return JSON.parse(await this.getLintDescriptionsHTMLAsJSON()) as Record<string, string>;
	}

	getDefaultLintConfigAsJSON(): Promise<string> {
		return this.rpc('getDefaultLintConfigAsJSON', []);
	}

	async getDefaultLintConfig(): Promise<LintConfig> {
		return JSON.parse(await this.getDefaultLintConfigAsJSON()) as LintConfig;
	}

	ignoreLint(source: string, lint: Lint): Promise<void> {
		return this.rpc('ignoreLint', [source, lint]);
	}

	ignoreLintHash(hash: bigint): Promise<void> {
		return this.rpc('ignoreLintHash', [hash]);
	}

	exportIgnoredLints(): Promise<string> {
		return this.rpc('exportIgnoredLints', []);
	}

	importIgnoredLints(json: string): Promise<void> {
		return this.rpc('importIgnoredLints', [json]);
	}

	contextHash(source: string, lint: Lint): Promise<bigint> {
		return this.rpc('contextHash', [source, lint]);
	}

	clearIgnoredLints(): Promise<void> {
		return this.rpc('clearIgnoredLints', []);
	}

	importWords(words: string[]): Promise<void> {
		return this.rpc('importWords', [words]);
	}

	exportWords(): Promise<string[]> {
		return this.rpc('exportWords', []);
	}

	getDialect(): Promise<Dialect> {
		return this.rpc('getDialect', []);
	}

	setDialect(dialect: Dialect): Promise<void> {
		return this.rpc('setDialect', [dialect]);
	}

	summarizeStats(start?: bigint, end?: bigint): Promise<any> {
		return this.rpc('summarizeStats', [start, end]);
	}

	generateStatsFile(): Promise<string> {
		return this.rpc('generateStatsFile', []);
	}

	importStatsFile(statsFile: string): Promise<void> {
		return this.rpc('importStatsFile', [statsFile]);
	}

	/** Run a procedure on the remote worker. */
	private async rpc(procName: string, args: unknown[]): Promise<any> {
		const promise = new Promise((resolve, reject) => {
			this.requestQueue.push({
				resolve,
				reject,
				request: { procName, args },
			});

			this.submitRemainingRequests();
		});

		return promise;
	}

	private async submitRemainingRequests() {
		if (this.working) {
			return;
		}

		this.working = true;

		if (this.requestQueue.length > 0) {
			const { request } = this.requestQueue[0];
			const serialized = await this.binary.serialize(request);
			this.worker.postMessage(serialized);
		} else {
			this.working = false;
		}
	}
}
