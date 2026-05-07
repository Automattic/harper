export type SectionId =
  | "getting-started"
  | "general"
  | "writing"
  | "dictionary"
  | "shortcuts"
  | "rules"
  | "weirpacks"
  | "integrations"
  | "about";

export interface NavItem {
  id: SectionId;
  label: string;
  gradient: string;
}

export const MAIN_NAV_ITEMS: NavItem[] = [
  {
    id: "getting-started",
    label: "Getting Started",
    gradient: "linear-gradient(180deg, #f07a3a 0%, #c94614 100%)",
  },
  {
    id: "general",
    label: "General",
    gradient: "linear-gradient(180deg, #3e8cff 0%, #1a5dd9 100%)",
  },
  {
    id: "writing",
    label: "Writing",
    gradient: "linear-gradient(180deg, #e48a4a 0%, #b06a1b 100%)",
  },
  {
    id: "dictionary",
    label: "Dictionary",
    gradient: "linear-gradient(180deg, #f5a0c0 0%, #c75c9c 100%)",
  },
  {
    id: "shortcuts",
    label: "Shortcuts",
    gradient: "linear-gradient(180deg, #f3c34a 0%, #d98b22 100%)",
  },
  {
    id: "rules",
    label: "Rules",
    gradient: "linear-gradient(180deg, #4cc28b 0%, #1f8f5c 100%)",
  },
  {
    id: "weirpacks",
    label: "Weirpacks",
    gradient: "linear-gradient(180deg, #b78cf6 0%, #7c3aed 100%)",
  },
  {
    id: "integrations",
    label: "Integrations",
    gradient: "linear-gradient(180deg, #6b6f78 0%, #3b3f48 100%)",
  },
];

export const FOOTER_NAV_ITEMS: NavItem[] = [
  {
    id: "about",
    label: "About",
    gradient: "linear-gradient(180deg, #8dd1ff 0%, #4aa2e0 100%)",
  },
];

export type AccessibilityState = "not_granted" | "granted";
export type IntegrationSetupState = "not_selected" | "selected";
export type TestDriveState = "not_started" | "completed";

export interface SetupState {
  accessibility: AccessibilityState;
  integration: IntegrationSetupState;
  testDrive: TestDriveState;
}

export type RuleOverride = "default" | "on" | "off" | "require" | "forbid";

export interface RuleStateOption {
  value: RuleOverride;
  label: string;
}

export interface RuleItem {
  id: string;
  name: string;
  desc: string;
  states?: RuleStateOption[];
}

export interface RuleGroup {
  id: string;
  title: string;
  desc: string;
  rules: RuleItem[];
}

export interface Weirpack {
  id: string;
  name: string;
  filename: string;
  size: number;
  enabled: boolean;
  ruleCount: number;
  addedAt: string;
  installState: "installed" | "installing" | "failed";
}

export interface AppIntegration {
  id: string;
  name: string;
  kind: string;
  tint: string;
  note?: string;
  custom?: boolean;
}

export interface IntegrationMetadata {
  name: string;
  tint: string;
  note?: string;
}

export const CURATED_INTEGRATION_METADATA: Record<string, IntegrationMetadata> = {
  "com.apple.TextEdit": {
    name: "TextEdit",
    tint: "#5a5f68",
    note: "Built into macOS and useful for trying Harper.",
  },
  "com.apple.mail": { name: "Mail", tint: "#3a8bff" },
  "com.apple.MobileSMS": { name: "Messages", tint: "#2dc652" },
  "com.apple.Notes": { name: "Notes", tint: "#f6c444" },
};

export interface SettingsState {
  setup: SetupState;
  menuBar: boolean;
  menuBarClick: "open-settings" | "show-menu" | "toggle-pause" | "quick-review";
  launchAtStartup: boolean;
  autoUpdate: boolean;
  dialect: string;
  strictness: "relaxed" | "standard" | "strict";
  liveCheck: boolean;
  respectCode: boolean;
  dictionary: string[];
  globalShortcuts: boolean;
  activationKey: "off" | "option" | "control" | "shift";
  rules: Record<string, RuleOverride>;
  integrations: Record<string, boolean>;
  watchEverywhere: boolean;
  autoIntegrate: boolean;
  customApps: AppIntegration[];
  removedBuiltins: string[];
  weirpacks: Weirpack[];
}

export const DIALECT_OPTIONS = [
  { value: "american", label: "American English" },
  { value: "british", label: "British English" },
  { value: "canadian", label: "Canadian English" },
  { value: "australian", label: "Australian English" },
  { value: "indian", label: "Indian English" },
];

export const RULE_GROUPS: RuleGroup[] = [
  {
    id: "proper-nouns",
    title: "Proper Nouns",
    desc: "Names of places, organizations, products, and brands that should keep standard capitalization.",
    rules: [
      {
        id: "pn-wordpress",
        name: "WordPress",
        desc: 'Always written as "WordPress", not "Wordpress" or "wordpress".',
      },
      {
        id: "pn-macos",
        name: "macOS",
        desc: 'Lowercase "m", uppercase "OS".',
      },
      {
        id: "pn-github",
        name: "GitHub",
        desc: 'Capital "G" and "H".',
      },
      {
        id: "pn-openai",
        name: "OpenAI",
        desc: "One word, capital O and AI.",
      },
    ],
  },
  {
    id: "initialisms",
    title: "Initialisms",
    desc: "Abbreviated forms that should use expected letter casing or punctuation.",
    rules: [
      { id: "in-api", name: "API", desc: "Always uppercase." },
      { id: "in-url", name: "URL", desc: "Always uppercase." },
      { id: "in-json", name: "JSON", desc: "Always uppercase." },
      { id: "in-etc", name: "etc.", desc: "Lowercase with a trailing period." },
    ],
  },
  {
    id: "phrase-corrections",
    title: "Phrase Corrections",
    desc: "Common phrase-level fixes where a standard expression is preferred.",
    rules: [
      {
        id: "ph-could-of",
        name: '"could of" -> "could have"',
        desc: 'Use "could have", never "could of".',
      },
      {
        id: "ph-alot",
        name: '"alot" -> "a lot"',
        desc: '"A lot" is always two words.',
      },
      {
        id: "ph-in-order-to",
        name: '"in order to" -> "to"',
        desc: 'Usually "to" is sufficient.',
      },
      {
        id: "ph-utilize",
        name: '"utilize" -> "use"',
        desc: "Prefer the shorter word.",
      },
    ],
  },
  {
    id: "punctuation",
    title: "Punctuation",
    desc: "Commas, periods, semicolons, and other marks.",
    rules: [
      {
        id: "pn-oxford",
        name: "Oxford comma",
        desc: "In a list of three or more, configure the comma before and/or.",
        states: [
          { value: "default", label: "Default" },
          { value: "require", label: "Require" },
          { value: "forbid", label: "Forbid" },
          { value: "off", label: "Off" },
        ],
      },
      {
        id: "pu-double-space",
        name: "Double space after period",
        desc: "Use only one space between sentences.",
      },
      {
        id: "pu-comma-splice",
        name: "Comma splice",
        desc: "Two independent clauses joined by only a comma.",
      },
    ],
  },
  {
    id: "clarity",
    title: "Clarity",
    desc: "Wordy, vague, or redundant constructions that can usually be tightened.",
    rules: [
      {
        id: "cl-wordy",
        name: "Wordy phrases",
        desc: "Tighten bloated expressions.",
      },
      {
        id: "cl-nominalization",
        name: "Nominalizations",
        desc: '"Make an assessment" -> "assess".',
      },
      {
        id: "cl-very",
        name: '"very" as intensifier',
        desc: "Prefer a stronger single word.",
      },
    ],
  },
];

export const ALL_RULES = RULE_GROUPS.flatMap((group) =>
  group.rules.map((rule) => ({ ...rule, groupId: group.id, groupTitle: group.title })),
);

export function createInitialSettingsState(): SettingsState {
  return {
    setup: {
      accessibility: "not_granted",
      integration: "not_selected",
      testDrive: "not_started",
    },
    menuBar: true,
    menuBarClick: "open-settings",
    launchAtStartup: true,
    autoUpdate: true,
    dialect: "american",
    strictness: "standard",
    liveCheck: true,
    respectCode: true,
    dictionary: [
      "HAL",
      "Harper",
      "Automattic",
      "WordPress",
      "macOS",
      "kubernetes",
      "LLM",
      "OKR",
      "SQLite",
      "OAuth",
      "webhooks",
      "dogfooding",
    ],
    globalShortcuts: true,
    activationKey: "off",
    rules: {},
    integrations: {
      textedit: false,
      mail: false,
      messages: false,
      notes: false,
    },
    watchEverywhere: false,
    autoIntegrate: false,
    customApps: [],
    removedBuiltins: [],
    weirpacks: [
      {
        id: "pack-001",
        name: "Internal Style Guide",
        filename: "automattic-style.weirpack",
        size: 18420,
        enabled: true,
        ruleCount: 47,
        addedAt: "2026-02-12T00:00:00Z",
        installState: "installed",
      },
      {
        id: "pack-002",
        name: "Plain English",
        filename: "plain-english.weirpack",
        size: 9120,
        enabled: true,
        ruleCount: 23,
        addedAt: "2026-03-04T00:00:00Z",
        installState: "installed",
      },
      {
        id: "pack-003",
        name: "Legalese Killer",
        filename: "legalese-killer.weirpack",
        size: 31580,
        enabled: false,
        ruleCount: 62,
        addedAt: "2026-04-18T00:00:00Z",
        installState: "installed",
      },
    ],
  };
}
