import { boolean, int, mysqlTable, text, timestamp } from 'drizzle-orm/mysql-core';

export const uninstallFeedbackTable = mysqlTable('uninstall_feedback', {
	id: int().autoincrement().primaryKey(),
	feedback: text().notNull(),
	timestamp: timestamp().notNull().defaultNow(),
});

export const problematicLintTable = mysqlTable('problematic_lint', {
	id: int().autoincrement().primaryKey(),
	/** If false, implied to be a false-negative. */
	is_false_positive: boolean().notNull(),
	example: text().notNull(),
	feedback: text().notNull(),
	rule_id: text(),
	timestamp: timestamp().notNull().defaultNow(),
});
