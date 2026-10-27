import { and, gte, lte } from 'drizzle-orm';
import { createInsertSchema, createSelectSchema } from 'drizzle-zod';
import { countReportsByDay } from '$lib/adminUtils';
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

	public static async getAllSince(date: Date) {
		return await db
			.select()
			.from(uninstallFeedbackTable)
			.where(gte(uninstallFeedbackTable.timestamp, date));
	}

	/**
	 * Count one uninstall reason's reports by UTC day, including zero-count days.
	 * Start is inclusive and end is exclusive; both must be UTC midnight.
	 * A null start uses the first matching report's day, or returns no points if none exist.
	 */
	public static async getDailyReportCounts(
		reason: string,
		start: Date | null,
		end: Date,
	): Promise<{ date: Date; value: number }[]> {
		const endTime = end.getTime();
		const reports = await this.getAllBetween(start ?? new Date(0), new Date(endTime - 1));
		return countReportsByDay(
			reports.filter((report) => (report.feedback ?? 'OTHER') === reason),
			start,
			end,
		);
	}

	public static async getAllBetween(start: Date, end: Date) {
		return await db
			.select()
			.from(uninstallFeedbackTable)
			.where(
				and(
					gte(uninstallFeedbackTable.timestamp, start),
					lte(uninstallFeedbackTable.timestamp, end),
				),
			);
	}
}
