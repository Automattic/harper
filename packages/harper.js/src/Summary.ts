/**
 * Represents the summary of linting results.
 */
export default interface Summary {
	/**
	 * An object mapping each lint type to its count.
	 * Example: `{ "Spelling": 4, "Capitalization": 1 }`
	 */
	lint_counts: Record<string, number>;

	/**
	 * The total number of fixes applied.
	 */
	total_applied: number;

	/**
	 * An object mapping misspelled words to their occurrence counts.
	 * Example: `{ "mispelled": 1, "mispell": 1, "thigs": 2 }`
	 */
	misspelled: Record<string, number>;
}
