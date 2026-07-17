
import { createInsertSchema, createSelectSchema } from 'drizzle-zod';
import { db } from '..';
import { problematicDomainTable } from '../schema';

export type ProblematicDomainRow = typeof problematicDomainTable.$inferSelect;
const ProblematicDomainRowParser = createSelectSchema(problematicDomainTable);

export type ProblematicDomainSubmission = typeof problematicDomainTable.$inferInsert;
const ProblematicDomainSubmissionParser = createInsertSchema(problematicDomainTable);

export default class ProblematicDomains {
	public static async validateAndCreate(rec: any) {
		const parsed = ProblematicDomainSubmissionParser.parse(rec);
		await this.create(parsed);
	}

	public static async create(rec: ProblematicDomainSubmission) {
		await db.insert(problematicDomainTable).values(rec);
	}
}
