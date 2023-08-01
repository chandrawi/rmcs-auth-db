CREATE TABLE "user" (
  "user_id" uuid NOT NULL,
  "name" varchar(64) NOT NULL,
  "email" varchar(64) NOT NULL,
  "phone" varchar(32) NOT NULL,
  "password" varchar(128) NOT NULL,
  "public_key" bytea NOT NULL,
  "private_key" bytea NOT NULL,
  PRIMARY KEY ("user_id"),
  UNIQUE ("name")
);

CREATE TABLE "user_role" (
  "user_id" uuid NOT NULL,
  "role_id" uuid NOT NULL,
  PRIMARY KEY ("user_id","role_id"),
  FOREIGN KEY ("user_id")
    REFERENCES "user" ("user_id"),
  FOREIGN KEY ("role_id")
    REFERENCES "role" ("role_id")
);
