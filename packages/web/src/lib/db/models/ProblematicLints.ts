import { and, gte, lte } from 'drizzle-orm';
import { createInsertSchema, createSelectSchema } from 'drizzle-zod';
import { db } from '..';
import { problematicLintTable } from '../schema';

export type ProblematicLintRow = typeof problematicLintTable.$inferSelect;
const ProblematicLintRowParser = createSelectSchema(problematicLintTable);

export type ProblematicLintSubmission = typeof problematicLintTable.$inferInsert;
const ProblematicLintSubmissionParser = createInsertSchema(problematicLintTable);

export default class ProblematicLints {
	public static async validateAndCreate(rec: any) {
		const parsed = ProblematicLintSubmissionParser.parse(rec);
		await this.create(parsed);
	}

	public static async create(rec: ProblematicLintSubmission) {
		await db.insert(problematicLintTable).values(rec);
	}

	public static async getAll(): Promise<ProblematicLintRow[]> {
		return await db.select().from(problematicLintTable);
	}

	public static async getAllSince(date: Date) {
		return await db
			.select()
			.from(problematicLintTable)
			.where(gte(problematicLintTable.timestamp, date));
	}

	public static async getDailyReportCounts(
		lintId: string,
		start: Date | null,
		end: Date,
	): Promise<{ date: Date; value: number }[]> {
		const endTime = end.getTime();
		const reports = await this.getAllBetween(start ?? new Date(0), new Date(endTime - 1));
		const dayLength = 24 * 60 * 60 * 1000;
		const counts = new Map<number, number>();
		let firstDay = start?.getTime() ?? Number.POSITIVE_INFINITY;

		for (const report of reports) {
			if ((report.rule_id ?? 'OTHER') !== lintId) continue;

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
			.from(problematicLintTable)
			.where(
				and(gte(problematicLintTable.timestamp, start), lte(problematicLintTable.timestamp, end)),
			);
	}
}
