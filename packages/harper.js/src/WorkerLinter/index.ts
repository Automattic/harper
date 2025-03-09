import type { Lint, Suggestion, Span } from 'harper-wasm';
import Linter, { LinterInit } from '../Linter';
import Worker from './worker.ts?worker&inline';
import { LintConfig, LintOptions } from '../main';
import { BinaryModule, DeserializedRequest } from '../binary';

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
	private worker: Worker;
	private requestQueue: RequestItem[];
	private working = true;

	constructor(init: LinterInit) {
		this.binary = init.binary;
		this.worker = new Worker();
		this.requestQueue = [];

		// Fires when the worker sends 'ready'.
		this.worker.onmessage = () => {
			this.setupMainEventListeners();

			this.worker.postMessage(this.binary.url);

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

	applySuggestion(text: string, suggestion: Suggestion, span: Span): Promise<string> {
		return this.rpc('applySuggestion', [text, suggestion, span]);
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

	getDefaultLintConfigAsJSON(): Promise<string> {
		return this.rpc('getDefaultLintConfigAsJSON', []);
	}

	async getDefaultLintConfig(): Promise<LintConfig> {
		return JSON.parse(await this.getDefaultLintConfigAsJSON()) as LintConfig;
	}

	async ignoreLint(lint: Lint): Promise<void> {
		return this.rpc('ignoreLint', [lint]);
	}

	async exportIgnoredLints(): Promise<string> {
		return this.rpc('exportIgnoredLints', []);
	}

	async importIgnoredLints(json: string): Promise<void> {
		return this.rpc('importIgnoredLints', [json]);
	}

	async clearIgnoredLints(): Promise<void> {
		return this.rpc('clearIgnoredLints', []);
	}

	async importWords(words: string[]): Promise<void> {
		return this.rpc('importWords', [words]);
	}

	async exportWords(): Promise<string[]> {
		return this.rpc('exportWords', []);
	}

	/** Run a procedure on the remote worker. */
	private async rpc(procName: string, args: any[]): Promise<any> {
		const promise = new Promise((resolve, reject) => {
			this.requestQueue.push({
				resolve,
				reject,
				request: { procName, args }
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
