export const isWindows: boolean =
	typeof navigator !== 'undefined' &&
	(/win/i.test((navigator as any).userAgentData?.platform || navigator.platform || '') ||
		/windows/i.test(navigator.userAgent || ''));

export const isMac: boolean =
	typeof navigator !== 'undefined' &&
	/mac/i.test((navigator as any).userAgentData?.platform || navigator.platform || '');

export const platformAppName: string = isWindows ? 'Notepad' : 'TextEdit';
export const platformAppId: string = isWindows ? 'notepad.exe' : 'com.apple.TextEdit';
export const platformPrivacyText: string = isWindows ? 'this PC' : 'this Mac';
export const platformTrayName: string = isWindows ? 'system tray' : 'menu bar';
export const platformAccessibilityName: string = isWindows
	? 'Windows UI Automation'
	: 'macOS Accessibility';
