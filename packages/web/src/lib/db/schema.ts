import { int, mysqlTable, text, timestamp } from 'drizzle-orm/mysql-core';

export const uninstallFeedbackTable = mysqlTable('uninstall_feedback', {
	id: int().autoincrement().primaryKey(),
	feedback: text().notNull(),
	timestamp: timestamp().notNull().defaultNow(),
});
