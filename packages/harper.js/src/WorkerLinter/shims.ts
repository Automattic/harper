if (import.meta.env.MODE === 'test') {
	// @ts-expect-error
	globalThis.__vitest_browser_runner__ = { wrapDynamicImport: (f) => f() };
}
