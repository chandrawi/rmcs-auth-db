CREATE TABLE `token` (
  `access_id` int(10) UNSIGNED NOT NULL,
  `refresh_id` char(32) NOT NULL,
  `user_id` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `expire` timestamp NULL DEFAULT current_timestamp(),
  `ip` varbinary(16) NOT NULL DEFAULT '\0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

ALTER TABLE `token`
  ADD PRIMARY KEY (`access_id`),
  ADD UNIQUE KEY `refresh_id` (`refresh_id`);
