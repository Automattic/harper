CREATE TABLE `domain_review` (
	`id` int AUTO_INCREMENT NOT NULL,
	`works` boolean NOT NULL,
	`feedback` text NOT NULL,
	`domain` text NOT NULL,
	`timestamp` timestamp NOT NULL DEFAULT (now()),
	CONSTRAINT `domain_review_id` PRIMARY KEY(`id`)
);
