import { invoke } from "@tauri-apps/api/core";
import { Dialect, type Lint, type LintConfig, type Linter } from "harper.js";

export class Client {
  static async getLintConfig(): Promise<LintConfig> {
    return await invoke<LintConfig>("get_lint_config");
  }

  static async getDialect(): Promise<Dialect> {
    return await invoke<Dialect>("get_dialect");
  }

  static async setDialect(dialect: Dialect): Promise<void> {
    await invoke("set_dialect", { dialect });
  }

  static async setLintConfig(lintConfig: LintConfig): Promise<void> {
    await invoke("set_lint_config", { lintConfig });
  }

  static async disableRule(ruleName: string): Promise<LintConfig> {
    const lintConfig = await Client.getLintConfig();
    lintConfig[ruleName] = false;

    await Client.setLintConfig(lintConfig);

    return lintConfig;
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
