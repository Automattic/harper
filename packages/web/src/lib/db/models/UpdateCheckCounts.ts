import { eq, sql } from 'drizzle-orm';
import { createSelectSchema } from 'drizzle-zod';
import { db } from '..';
import { updateCheckCountTable } from '../schema';

export type UpdateCheckCountRow = typeof updateCheckCountTable.$inferSelect;
const UpdateCheckCountRowParser = createSelectSchema(updateCheckCountTable);

export default class UpdateCheckCounts {
	public static async incrementForToday() {
		UpdateCheckCounts.incrementForDate(new Date());
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

	public static async getAll(): Promise<UpdateCheckCountRow[]> {
		return await db.select().from(updateCheckCountTable);
	}
}
