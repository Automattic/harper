import { invoke } from "@tauri-apps/api/core";
import { Dialect, type Lint, type LintConfig, type Linter } from "harper.js";

type RustDialect = "American" | "British" | "Australian" | "Canadian" | "Indian";

export class Client {
  static async getLintConfig(): Promise<LintConfig> {
    return await invoke<LintConfig>("get_lint_config");
  }

  static async getDialect(): Promise<Dialect> {
    return rustDialectToDialect(await invoke<RustDialect>("get_dialect"));
  }

  static async setDialect(dialect: Dialect): Promise<void> {
    await invoke("set_dialect", { dialect: dialectToRustDialect(dialect) });
  }

  static async setLintConfig(lintConfig: LintConfig): Promise<void> {
    await invoke("set_lint_config", { lintConfig });
  }

  static async getDictionary(): Promise<string[]> {
    return await invoke<string[]>("get_dictionary");
  }

  static async setDictionary(words: string[]): Promise<void> {
    await invoke("set_dictionary", { words });
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

  static async getAllowedBundleIdentifiers(): Promise<string[]> {
    return await invoke<string[]>("get_allowed_bundle_identifiers");
  }

  static async addAllowedBundleIdentifier(bundleIdentifier: string): Promise<void> {
    await invoke("add_allowed_bundle_identifier", { bundleIdentifier });
  }

  static async removeAllowedBundleIdentifier(bundleIdentifier: string): Promise<void> {
    await invoke("remove_allowed_bundle_identifier", { bundleIdentifier });
  }
}

function rustDialectToDialect(dialect: RustDialect): Dialect {
  switch (dialect) {
    case "British":
      return Dialect.British;
    case "Australian":
      return Dialect.Australian;
    case "Canadian":
      return Dialect.Canadian;
    case "Indian":
      return Dialect.Indian;
    case "American":
    default:
      return Dialect.American;
  }
}

function dialectToRustDialect(dialect: Dialect): RustDialect {
  switch (dialect) {
    case Dialect.British:
      return "British";
    case Dialect.Australian:
      return "Australian";
    case Dialect.Canadian:
      return "Canadian";
    case Dialect.Indian:
      return "Indian";
    case Dialect.American:
    default:
      return "American";
  }
}
