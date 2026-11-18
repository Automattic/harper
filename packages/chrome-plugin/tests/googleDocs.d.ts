/** The Google Docs annotated-text API used by the live integration tests. */
interface GoogleDocsAnnotatedText {
	getText(): string;
	setSelection(start: number, end: number): void;
	getSelection?: () => Record<string, unknown>[];
}

interface Window {
	/** Google Docs installs this API asynchronously when its editor loads. */
	_docs_annotate_getAnnotatedText?: (extensionId?: string) => Promise<GoogleDocsAnnotatedText>;
}
