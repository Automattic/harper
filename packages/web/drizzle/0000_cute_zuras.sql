CREATE TABLE `uninstall_feedback` (
	`id` int AUTO_INCREMENT NOT NULL,
	`feedback` text NOT NULL,
	`timestamp` timestamp NOT NULL DEFAULT (now()),
	CONSTRAINT `uninstall_feedback_id` PRIMARY KEY(`id`)
);
