import { and, gte, lte } from 'drizzle-orm';
import { createInsertSchema, createSelectSchema } from 'drizzle-zod';
import { db } from '..';
import { domainReviewTable } from '../schema';

export type DomainReviewRow = typeof domainReviewTable.$inferSelect;
const DomainReviewRowParser = createSelectSchema(domainReviewTable);

export type DomainReviewSubmission = typeof domainReviewTable.$inferInsert;
const DomainReviewSubmissionParser = createInsertSchema(domainReviewTable);

export default class DomainReviews {
	public static async validateAndCreate(rec: any) {
		const parsed = DomainReviewSubmissionParser.parse(rec);
		await this.create(parsed);
	}

	public static async create(rec: DomainReviewSubmission) {
		await db.insert(domainReviewTable).values(rec);
	}

	public static async getAllSince(date: Date) {
		return await db.select().from(domainReviewTable).where(gte(domainReviewTable.timestamp, date));
	}

	/**
	 * Count one domain's reports by UTC day, including zero-count days.
	 * Start is inclusive and end is exclusive; both must be UTC midnight.
	 * A null start uses the first matching report's day, or returns no points if none exist.
	 */
	public static async getDailyReportCounts(
		domain: string,
		start: Date | null,
		end: Date,
	): Promise<{ date: Date; value: number }[]> {
		const endTime = end.getTime();
		const reports = await this.getAllBetween(start ?? new Date(0), new Date(endTime - 1));
		const dayLength = 24 * 60 * 60 * 1000;
		const counts = new Map<number, number>();
		let firstDay = start?.getTime() ?? Number.POSITIVE_INFINITY;

		for (const report of reports) {
			if ((report.domain ?? 'OTHER') !== domain) continue;

			const day = Math.floor(report.timestamp.getTime() / dayLength) * dayLength;
			counts.set(day, (counts.get(day) ?? 0) + 1);
			firstDay = Math.min(firstDay, day);
		}

		const points: { date: Date; value: number }[] = [];
		for (let day = firstDay; day < endTime; day += dayLength) {
			points.push({ date: new Date(day), value: counts.get(day) ?? 0 });
		}
		return points;
	}

	public static async getAllBetween(start: Date, end: Date) {
		return await db
			.select()
			.from(domainReviewTable)
			.where(and(gte(domainReviewTable.timestamp, start), lte(domainReviewTable.timestamp, end)));
	}
}
