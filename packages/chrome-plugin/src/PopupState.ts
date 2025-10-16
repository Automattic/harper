export type PopupState =
	| {
			page: 'onboarding';
	  }
	| {
			page: 'main';
	  }
	| {
			page: 'report-error';
			feedback: string;
			example: string;
			rule_id: string;
	  };

export function main(): PopupState {
	return { page: 'main' };
}
