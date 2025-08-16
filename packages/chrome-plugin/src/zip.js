import gulp from 'gulp';
import zip from 'gulp-zip';
import { createRequire } from 'module';

const require = createRequire(import.meta.url);
const manifest = require('../build/manifest.json');

const [, , target] = process.argv;
if (!target) {
	process.stderr.write('Specify a target filename as the first argument.\n');
	process.exit(1);
}

gulp.src('build/**', { encoding: false }).pipe(zip(target)).pipe(gulp.dest('package'));
