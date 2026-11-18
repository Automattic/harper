/**
 * Dark-mode overrides for the shared `components` Button inside Weir Studio.
 * The shared Button uses a cool gray dark palette; Weir Studio uses the site's warm palette.
 * Important modifiers are required because both class sets sit in the same utilities layer.
 */

/** Use with `color="white"`. */
export const studioSecondaryButton =
	'dark:bg-[#1c1a16]! dark:text-white/80! dark:border-white/15! dark:hover:bg-white/10! dark:focus:ring-white/20!';

/** Use with `color="dark"`. The button inverts to white in dark mode. */
export const studioPrimaryButton =
	'dark:bg-white! dark:text-black! dark:hover:bg-white/85! dark:focus:ring-white/30!';

/** Ace themes that match the light and dark site themes. */
export const studioAceTheme = { light: 'chrome', dark: 'tomorrow_night' } as const;
