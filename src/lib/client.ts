import { invoke } from "@tauri-apps/api/core";
import type { Lint, LintConfig, Linter } from "harper.js";

export class Client {
  static async getLintConfig(): Promise<LintConfig> {
    return await invoke<LintConfig>("get_lint_config");
  }

  static async ignoreLint(linter: Linter, source: string, lint: Lint): Promise<void> {
    await linter.ignoreLint(source, lint);
    const ignoredLints = await linter.exportIgnoredLints();

    await invoke("ignore_lint", { ignoredLints });
  }

  static async addToDictionary(word: string): Promise<void> {
    await invoke("add_to_dictionary", { word });
  }
}
