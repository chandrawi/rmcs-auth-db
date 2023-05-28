CREATE TABLE `token` (
  `refresh_id` char(32) NOT NULL,
  `access_id` char(8) NOT NULL,
  `user_id` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `expire` timestamp NULL DEFAULT current_timestamp(),
  `ip` varbinary(16) NOT NULL DEFAULT '\0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

ALTER TABLE `token`
  ADD PRIMARY KEY (`refresh_id`);
