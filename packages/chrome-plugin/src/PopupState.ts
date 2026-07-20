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
	  }
	| {
			page: 'report-domain';
			works: boolean;
			domain: string;
			feedback: string;
	  };

export function main(): PopupState {
	return { page: 'main' };
}
