CREATE TABLE `token` (
  `access_id` int(10) UNSIGNED NOT NULL,
  `user_id` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `refresh_token` char(32) NOT NULL,
  `auth_token` char(32) NOT NULL,
  `expire` timestamp NULL DEFAULT current_timestamp(),
  `ip` varbinary(16) NOT NULL DEFAULT '\0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

ALTER TABLE `token`
  ADD PRIMARY KEY (`access_id`);
