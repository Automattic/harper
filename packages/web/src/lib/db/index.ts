import { drizzle } from 'drizzle-orm/mysql2';

export const db = drizzle({ connection: { uri: process.env.DATABASE_URL } });
