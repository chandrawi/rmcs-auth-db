CREATE TABLE "token" (
  "access_id" int NOT NULL,
  "user_id" int NOT NULL DEFAULT 0,
  "refresh_token" char(32) NOT NULL,
  "auth_token" char(32) NOT NULL,
  "expire" timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "ip" bytea,
  PRIMARY KEY ("access_id")
);
