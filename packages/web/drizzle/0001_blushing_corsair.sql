CREATE TABLE `problematic_lint` (
	`id` int AUTO_INCREMENT NOT NULL,
	`is_false_positive` boolean NOT NULL,
	`example` text NOT NULL,
	`feedback` text NOT NULL,
	`timestamp` timestamp NOT NULL DEFAULT (now()),
	CONSTRAINT `problematic_lint_id` PRIMARY KEY(`id`)
);
