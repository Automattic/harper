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
}
