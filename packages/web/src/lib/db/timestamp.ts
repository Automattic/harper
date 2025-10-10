import { sql } from 'drizzle-orm';
import { int } from 'drizzle-orm/sqlite-core';

export default function timestamp() {
	return int('timestamp1', { mode: 'timestamp' }).notNull().default(sql`(unixepoch())`);
}
