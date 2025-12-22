import { defineConfig } from 'drizzle-kit';

export default defineConfig({
	out: './drizzle',
	schema: './src/lib/db/schema.ts',
	dialect: 'mysql',
	dbCredentials: {
		url: process.env.DATABASE_URL!,
	},
});
