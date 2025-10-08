import { int, sqliteTable, text } from 'drizzle-orm/sqlite-core';
import timestamp from './timestamp';

export const uninstallFeedbackTable = sqliteTable('uninstall_feedback', {
	id: int().primaryKey({ autoIncrement: true }),
	feedback: text().notNull(),
	timestamp: timestamp(),
});
