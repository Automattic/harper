import path from 'path';
import { createFixture } from 'playwright-webextext';

const pathToExtension = path.join(import.meta.dirname, '../build');
export const { test, expect } = createFixture(pathToExtension);
