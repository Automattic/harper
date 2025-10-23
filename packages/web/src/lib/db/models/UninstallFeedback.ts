import { createInsertSchema, createSelectSchema } from 'drizzle-zod';
import { db } from '..';
import { uninstallFeedbackTable } from '../schema';

export type UninstallFeedbackRow = typeof uninstallFeedbackTable.$inferSelect;
const UninstallFeedbackRowParser = createSelectSchema(uninstallFeedbackTable);

export type UninstallFeedbackSubmission = typeof uninstallFeedbackTable.$inferInsert;
const UninstallFeedbackSubmissionParser = createInsertSchema(uninstallFeedbackTable);

export default class UninstallFeedback {
	public static async validateAndCreate(rec: any) {
		const parsed = UninstallFeedbackSubmissionParser.parse(rec);
		await this.create(parsed);
	}

	public static async create(rec: UninstallFeedbackSubmission) {
		await db.insert(uninstallFeedbackTable).values(rec);
	}
}
