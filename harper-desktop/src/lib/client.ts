import { invoke } from "@tauri-apps/api/core";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { Dialect, type Lint, type LintConfig, type Linter, type StructuredLintConfig } from "harper.js";

type RustDialect = "American" | "British" | "Australian" | "Canadian" | "Indian";

export interface Integration {
  bundle_id: string;
  enabled: boolean;
}

export class Client {
  static async getLintConfig(): Promise<LintConfig> {
    return await invoke<LintConfig>("get_lint_config");
  }

  static async getDefaultLintConfig(): Promise<LintConfig> {
    return await invoke<LintConfig>("get_default_lint_config");
  }

  static async getStructuredLintConfig(): Promise<StructuredLintConfig> {
    const structuredLintConfig = await invoke<string>("get_structured_lint_config");
    console.debug("[harper-desktop] raw structured lint config", {
      length: structuredLintConfig.length,
      preview: structuredLintConfig.slice(0, 500),
    });

    const parsedStructuredLintConfig = JSON.parse(structuredLintConfig) as StructuredLintConfig;
    console.debug("[harper-desktop] parsed structured lint config", {
      keys: Object.keys(parsedStructuredLintConfig),
      settingsCount: parsedStructuredLintConfig.settings?.length,
      firstSetting: parsedStructuredLintConfig.settings?.[0],
    });

    return parsedStructuredLintConfig;
  }

  static async getDialect(): Promise<Dialect> {
    return rustDialectToDialect(await invoke<RustDialect>("get_dialect"));
  }

  static async setDialect(dialect: Dialect): Promise<void> {
    await invoke("set_dialect", { dialect: dialectToRustDialect(dialect) });
  }

  static async getLaunchAtStartup(): Promise<boolean> {
    return await isEnabled();
  }

  static async setLaunchAtStartup(enabled: boolean): Promise<void> {
    if (enabled) {
      await enable();
    } else {
      await disable();
    }
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

  static async getIntegrations(): Promise<Integration[]> {
    return await invoke<Integration[]>("get_integrations");
  }

  static async addIntegration(bundleId: string): Promise<void> {
    await invoke("add_integration", { bundleId });
  }

  static async removeIntegration(bundleId: string): Promise<void> {
    await invoke("remove_integration", { bundleId });
  }

  static async setIntegrationEnabled(bundleId: string, enabled: boolean): Promise<void> {
    await invoke("set_integration_enabled", { bundleId, enabled });
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
