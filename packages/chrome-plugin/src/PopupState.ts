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
