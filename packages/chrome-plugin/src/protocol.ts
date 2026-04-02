import type { Dialect, LintConfig, LintOptions } from 'harper.js';
import type { UnpackedLintGroups } from 'lint-framework';

export type Request =
	| LintRequest
	| GetConfigRequest
	| SetConfigRequest
	| GetLintDescriptionsRequest
	| SetDialectRequest
	| GetDialectRequest
	| SetDomainStatusRequest
	| SetDefaultStatusRequest
	| GetDomainStatusRequest
	| GetDefaultStatusRequest
	| GetEnabledDomainsRequest
	| AddToUserDictionaryRequest
	| SetUserDictionaryRequest
	| IgnoreLintRequest
	| GetUserDictionaryRequest
	| GetActivationKeyRequest
	| SetActivationKeyRequest
	| GetHotkeyRequest
	| SetHotkeyRequest
	| OpenOptionsRequest
	| GetInstalledOnRequest
	| GetReviewedRequest
	| SetReviewedRequest
	| OpenReportErrorRequest
	| PostFormDataRequest
	| GetWeirpacksRequest
	| AddWeirpackRequest
	| RemoveWeirpackRequest;

export type Response =
	| LintResponse
	| GetConfigResponse
	| UnitResponse
	| GetLintDescriptionsResponse
	| GetDialectResponse
	| GetDomainStatusResponse
	| GetDefaultStatusResponse
	| GetEnabledDomainsResponse
	| GetUserDictionaryResponse
	| GetHotkeyResponse
	| GetActivationKeyResponse
	| GetInstalledOnResponse
	| GetReviewedResponse
	| PostFormDataResponse
	| GetWeirpacksResponse;

export type LintRequest = {
	kind: 'lint';
	domain: string;
	text: string;
	options: LintOptions;
};

export type LintResponse = {
	kind: 'lints';
	lints: UnpackedLintGroups;
};

export type GetConfigRequest = {
	kind: 'getConfig';
};

export type GetConfigResponse = {
	kind: 'getConfig';
	config: LintConfig;
};

export type SetConfigRequest = {
	kind: 'setConfig';
	config: LintConfig;
};

export type SetDialectRequest = {
	kind: 'setDialect';
	dialect: Dialect;
};

export type GetLintDescriptionsRequest = {
	kind: 'getLintDescriptions';
};

export type GetLintDescriptionsResponse = {
	kind: 'getLintDescriptions';
	descriptions: Record<string, string>;
};

export type GetDialectRequest = {
	kind: 'getDialect';
};

export type GetDialectResponse = {
	kind: 'getDialect';
	dialect: Dialect;
};

export type GetDomainStatusRequest = {
	kind: 'getDomainStatus';
	domain: string;
};

export type GetDomainStatusResponse = {
	kind: 'getDomainStatus';
	domain: string;
	enabled: boolean;
};

export type GetDefaultStatusRequest = {
	kind: 'getDefaultStatus';
};

export type GetDefaultStatusResponse = {
	kind: 'getDefaultStatus';
	enabled: boolean;
};

export type GetEnabledDomainsRequest = {
	kind: 'getEnabledDomains';
};

export type GetEnabledDomainsResponse = {
	kind: 'getEnabledDomains';
	domains: string[];
};

export type SetDomainStatusRequest = {
	kind: 'setDomainStatus';
	domain: string;
	enabled: boolean;
	/** Dictates whether this should override a previous setting. */
	overrideValue: boolean;
};

export type SetDefaultStatusRequest = {
	kind: 'setDefaultStatus';
	enabled: boolean;
};

export type AddToUserDictionaryRequest = {
	kind: 'addToUserDictionary';
	words: string[];
};

export type SetUserDictionaryRequest = {
	kind: 'setUserDictionary';
	words: string[];
};

export type GetUserDictionaryRequest = {
	kind: 'getUserDictionary';
};

export type GetUserDictionaryResponse = {
	kind: 'getUserDictionary';
	words: string[];
};

export type GetInstalledOnRequest = {
	kind: 'getInstalledOn';
};

export type GetInstalledOnResponse = {
	kind: 'getInstalledOn';
	installedOn: string | null;
};

export type GetReviewedRequest = {
	kind: 'getReviewed';
};

export type GetReviewedResponse = {
	kind: 'getReviewed';
	reviewed: boolean;
};

export type SetReviewedRequest = {
	kind: 'setReviewed';
	reviewed: boolean;
};

export type IgnoreLintRequest = {
	kind: 'ignoreLint';
	contextHash: string;
};

/** Similar to returning void. */
export type UnitResponse = {
	kind: 'unit';
};

export function createUnitResponse(): UnitResponse {
	return { kind: 'unit' };
}

export enum ActivationKey {
	Off = 'off',
	Shift = 'shift',
	Control = 'control',
}

export type GetActivationKeyRequest = {
	kind: 'getActivationKey';
};

export type GetHotkeyRequest = {
	kind: 'getHotkey';
};

export type GetActivationKeyResponse = {
	kind: 'getActivationKey';
	key: ActivationKey;
};

export type PostFormDataResponse = {
	kind: 'postFormData';
	success: boolean;
};

export type SetActivationKeyRequest = {
	kind: 'setActivationKey';
	key: ActivationKey;
};

export type OpenOptionsRequest = {
	kind: 'openOptions';
};

export type GetHotkeyResponse = {
	kind: 'getHotkey';
	hotkey: Hotkey;
};

export type SetHotkeyRequest = {
	kind: 'setHotkey';
	hotkey: Hotkey;
};

export type Modifier = 'Ctrl' | 'Shift' | 'Alt';

export type Hotkey = {
	modifiers: Modifier[];
	key: string;
};
export type OpenReportErrorRequest = {
	kind: 'openReportError';
	example: string;
	rule_id: string;
	feedback: string;
};

export type PostFormDataRequest = {
	kind: 'postFormData';
	url: string;
	formData: Record<string, string>;
};

export type WeirpackMeta = {
	id: string;
	name: string;
	filename: string;
	version: string | null;
	installedAt: string;
};

export type GetWeirpacksRequest = {
	kind: 'getWeirpacks';
};

export type GetWeirpacksResponse = {
	kind: 'getWeirpacks';
	weirpacks: WeirpackMeta[];
};

export type AddWeirpackRequest = {
	kind: 'addWeirpack';
	filename: string;
	bytes: number[];
};

export type RemoveWeirpackRequest = {
	kind: 'removeWeirpack';
	id: string;
};
