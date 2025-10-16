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
}
