CREATE TABLE `auth_role` (
  `role_id` smallint(5) UNSIGNED NOT NULL,
  `role` varchar(64) NOT NULL,
  `secured` tinyint(1) NOT NULL DEFAULT 1,
  `multi` tinyint(1) NOT NULL DEFAULT 0,
  `token_expire` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `token_limit` int(10) UNSIGNED NOT NULL DEFAULT 0
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

CREATE TABLE `auth_access` (
  `role_id` smallint(5) UNSIGNED NOT NULL,
  `procedure_id` smallint(5) UNSIGNED NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

CREATE TABLE `auth_user` (
  `role_id` smallint(5) UNSIGNED NOT NULL,
  `user_id` int(10) UNSIGNED NOT NULL,
  `user` varchar(64) NOT NULL,
  `password` varchar(60) NOT NULL,
  `public_key` varchar(4096) NOT NULL,
  `private_key` varchar(4096) NOT NULL,
  `email` varchar(64) NOT NULL,
  `phone` varchar(32) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

CREATE TABLE `auth_token` (
  `id` char(32) NOT NULL,
  `role_id` smallint(5) UNSIGNED NOT NULL,
  `user_id` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `expire` timestamp NULL DEFAULT current_timestamp(),
  `limit` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `ip` varbinary(16) NOT NULL DEFAULT '\0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin;

ALTER TABLE `auth_role`
  ADD PRIMARY KEY (`role_id`),
  ADD UNIQUE KEY `role` (`role`);

ALTER TABLE `auth_access`
  ADD PRIMARY KEY (`role_id`,`procedure_id`),
  ADD KEY `access_role_id` (`role_id`),
  ADD KEY `access_procedure_id` (`procedure_id`);

ALTER TABLE `auth_user`
  ADD PRIMARY KEY (`user_id`),
  ADD UNIQUE KEY `user` (`user`),
  ADD KEY `user_role_id` (`role_id`);

ALTER TABLE `auth_token`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `auth_role`
  MODIFY `role_id` smallint(5) UNSIGNED NOT NULL AUTO_INCREMENT;

ALTER TABLE `auth_user`
  MODIFY `user_id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT;

ALTER TABLE `auth_access`
  ADD CONSTRAINT `access_procedure_id` FOREIGN KEY (`procedure_id`) REFERENCES `api_procedure` (`api_id`),
  ADD CONSTRAINT `access_role_id` FOREIGN KEY (`role_id`) REFERENCES `auth_role` (`role_id`);

ALTER TABLE `auth_user`
  ADD CONSTRAINT `user_role_id` FOREIGN KEY (`role_id`) REFERENCES `auth_role` (`role_id`);
