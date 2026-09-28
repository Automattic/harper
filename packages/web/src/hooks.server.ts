import { error, type Handle } from '@sveltejs/kit';
import { migrate } from 'drizzle-orm/mysql2/migrator';
import { ENABLE_ADMIN_ROUTES } from '$env/static/private';
import { db } from '$lib/db';

// Migrate the database exactly once at startup
try {
	await migrate(db, { migrationsFolder: './drizzle', migrationsTable: '__drizzle_migrations' });
} catch (e: any) {
	console.log('Failed to migrate database.');
	console.error(e);
}

export const handle: Handle = async ({ event, resolve }) => {
	const adminRoutesEnabled = ENABLE_ADMIN_ROUTES === 'true';

	if (!adminRoutesEnabled && event.route.id?.startsWith('/admin')) {
		error(404, 'Not found');
	}

	return resolve(event);
};
