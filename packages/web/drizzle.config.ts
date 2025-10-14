import 'dotenv/config';
import { defineConfig } from 'drizzle-kit';

export default defineConfig({
	out: './drizzle',
	schema: './src/lib/db/schema.ts',
	dialect: 'mysql',
	dbCredentials: {
		url: process.env.DB_FILE_NAME!,
	},
});
