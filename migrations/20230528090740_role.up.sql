CREATE TABLE `role` (
  `role_id` smallint(5) UNSIGNED NOT NULL,
  `api_id` smallint(5) UNSIGNED NOT NULL,
  `name` varchar(64) NOT NULL,
  `multi` tinyint(1) NOT NULL DEFAULT 1,
  `ip_lock` tinyint(1) NOT NULL DEFAULT 0,
  `access_duration` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `refresh_duration` int(10) UNSIGNED NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

CREATE TABLE `role_access` (
  `role_id` smallint(5) UNSIGNED NOT NULL,
  `procedure_id` int(10) UNSIGNED NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

ALTER TABLE `role`
  ADD PRIMARY KEY (`role_id`),
  ADD UNIQUE KEY `name` (`api_id`,`name`);

ALTER TABLE `role_access`
  ADD PRIMARY KEY (`role_id`,`procedure_id`),
  ADD KEY `access_role_id` (`role_id`),
  ADD KEY `access_procedure_id` (`procedure_id`);

ALTER TABLE `role`
  MODIFY `role_id` smallint(5) UNSIGNED NOT NULL AUTO_INCREMENT;

ALTER TABLE `role`
  ADD CONSTRAINT `role_api_id` FOREIGN KEY (`api_id`) REFERENCES `api` (`api_id`);

ALTER TABLE `role_access`
  ADD CONSTRAINT `access_role_id` FOREIGN KEY (`role_id`) REFERENCES `role` (`role_id`),
  ADD CONSTRAINT `access_procedure_id` FOREIGN KEY (`procedure_id`) REFERENCES `api_procedure` (`procedure_id`);
