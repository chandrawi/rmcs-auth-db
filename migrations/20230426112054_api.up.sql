CREATE TABLE `api` (
  `api_id` smallint(5) UNSIGNED NOT NULL,
  `name` varchar(64) NOT NULL,
  `address` varchar(128) NOT NULL,
  `category` varchar(64) NOT NULL,
  `password` varchar(128) NOT NULL,
  `public_key` varchar(4096) NOT NULL,
  `private_key` varchar(4096) NOT NULL,
  `description` varchar(255) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

CREATE TABLE `api_procedure` (
  `procedure_id` int(10) UNSIGNED NOT NULL,
  `api_id` smallint(5) UNSIGNED NOT NULL,
  `name` varchar(64) NOT NULL,
  `description` varchar(255) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

ALTER TABLE `api`
  ADD PRIMARY KEY (`api_id`),
  ADD UNIQUE KEY `name` (`name`);

ALTER TABLE `api_procedure`
  ADD PRIMARY KEY (`procedure_id`),
  ADD UNIQUE KEY `api_procedure` (`api_id`,`name`);

ALTER TABLE `api`
  MODIFY `api_id` smallint(5) UNSIGNED NOT NULL AUTO_INCREMENT;

ALTER TABLE `api_procedure`
  MODIFY `procedure_id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT;

ALTER TABLE `api_procedure`
  ADD CONSTRAINT `api_procedure_id` FOREIGN KEY (`api_id`) REFERENCES `api` (`api_id`);
