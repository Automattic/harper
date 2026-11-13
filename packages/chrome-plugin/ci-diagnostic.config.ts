import path from "node:path";
import { defineConfig } from "@playwright/test";
import original from "./playwright.config";

const tools =
	process.env.CI_DIAGNOSTIC_TOOLS ||
	path.join(process.env.HOME!, "bin/harper-ci-diagnostic/node_modules");

export default defineConfig({
	...original,
	testDir: "./tests",
	testMatch: ["lexical.spec.ts", "iframe_domain_inheritance.spec.ts"],
	grep: /Can apply basic suggestion\.|inherits when the parent is `www\.`|preserves an explicit iframe opt-in/,
	repeatEach: 50,
	retries: 0,
	workers: 2,
	globalTimeout: 1200000,
	outputDir: "diagnostic-output/test-results",
	reporter: [
		["line"],
		["json", { outputFile: "diagnostic-output/results.json" }],
	],
	projects: original.projects?.filter((project) => project.name === "chromium"),
	webServer: {
		...original.webServer,
		command: `node ${JSON.stringify(path.join(tools, "http-server/bin/http-server"))} ./tests/pages -p 8081 -a 127.0.0.1`,
		reuseExistingServer: false,
	},
});
