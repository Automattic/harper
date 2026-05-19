import { describe, it } from "node:test";
import assert from "node:assert";
import { readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const manifest = JSON.parse(
	readFileSync(resolve(__dirname, "..", "manifest.json"), "utf-8"),
);

describe("manifest.json content_scripts matches", () => {
	it("should have content_scripts defined", () => {
		assert.ok(Array.isArray(manifest.content_scripts));
		assert.ok(manifest.content_scripts.length > 0);
	});

	it("should have matches patterns that cover https://*/*", () => {
		const matches = manifest.content_scripts[0].matches;
		assert.ok(matches.includes("https://*/*"));
	});

	it("should cover write.ellipsus.com via broad https pattern", () => {
		const matches = manifest.content_scripts[0].matches;
		const coversEllipsus = matches.some((pattern) => {
			if (pattern === "https://*/*" || pattern === "<all_urls>") return true;
			if (pattern === "https://*.ellipsus.com/*") return true;
			if (pattern === "https://write.ellipsus.com/*") return true;
			return false;
		});
		assert.ok(coversEllipsus, "matches should cover Ellipsus domains");
	});

	it("should cover arbitrary https web apps", () => {
		const matches = manifest.content_scripts[0].matches;
		const coversBroad = matches.includes("https://*/*") || matches.includes("<all_urls>");
		assert.ok(coversBroad, "matches should use a broad pattern for general web app support");
	});

	it("should also cover http sites", () => {
		const matches = manifest.content_scripts[0].matches;
		assert.ok(matches.includes("http://*/*"));
	});
});
