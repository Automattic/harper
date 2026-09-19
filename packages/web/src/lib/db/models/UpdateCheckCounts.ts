import { eq, sql } from 'drizzle-orm';
import { createSelectSchema } from 'drizzle-zod';
import { db } from '..';
import { updateCheckCountTable } from '../schema';

export type UpdateCheckCountRow = typeof updateCheckCountTable.$inferSelect;
const UpdateCheckCountRowParser = createSelectSchema(updateCheckCountTable);

export default class DomainReviews {
	public static async incrementForToday() {
		DomainReviews.incrementForDate(new Date());
	}

	public static async incrementForDate(date: Date) {
		await db
			.insert(updateCheckCountTable)
			.values({
				date: date,
				count: 1,
			})
			.onDuplicateKeyUpdate({
				set: {
					count: sql`${updateCheckCountTable.count} + 1`,
				},
			});
	}

	public static async getCountForDate(date: Date): Promise<number> {
		const found = await db
			.select()
			.from(updateCheckCountTable)
			.where(eq(updateCheckCountTable.date, date));

		const first = found[0];

		if (first == null) {
			return 0;
		} else {
			return first.count;
		}
	}
}
