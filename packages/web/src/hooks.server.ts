import { db } from '$lib/db';
import { migrate } from 'drizzle-orm/mysql2/migrator';

// Migrate exactly once at startup
migrate(db, { migrationsFolder: './drizzle', migrationsTable: '__drizzle_migrations' });
