import { migrate } from 'drizzle-orm/mysql2/migrator';
import { db } from '$lib/db';

// Migrate exactly once at startup
try {
	await migrate(db, { migrationsFolder: './drizzle', migrationsTable: '__drizzle_migrations' });
} catch (e: any) {
	console.log('Failed to migrate database.');
	console.error(e);
}
