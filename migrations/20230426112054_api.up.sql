CREATE TABLE `api` (
  `api_id` smallint(5) UNSIGNED NOT NULL,
  `api_name` varchar(32) NOT NULL,
  `kind` enum('RESOURCE','APPLICATION') NOT NULL,
  `address` varchar(64) NOT NULL,
  `description` varchar(255) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

CREATE TABLE `api_procedure` (
  `api_id` smallint(5) UNSIGNED NOT NULL,
  `procedure_id` smallint(5) UNSIGNED NOT NULL,
  `service` varchar(64) NOT NULL,
  `procedure` varchar(64) NOT NULL,
  `description` varchar(255) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

ALTER TABLE `api`
  ADD PRIMARY KEY (`api_id`),
  ADD UNIQUE KEY `api_name` (`api_name`);

ALTER TABLE `api_procedure`
  ADD PRIMARY KEY (`procedure_id`),
  ADD UNIQUE KEY `api_procedure` (`api_id`,`service`,`procedure`);

ALTER TABLE `api`
  MODIFY `api_id` smallint(5) UNSIGNED NOT NULL AUTO_INCREMENT;

ALTER TABLE `api_procedure`
  MODIFY `procedure_id` smallint(5) UNSIGNED NOT NULL AUTO_INCREMENT;

ALTER TABLE `api_procedure`
  ADD CONSTRAINT `api_procedure_id` FOREIGN KEY (`api_id`) REFERENCES `api` (`api_id`);
