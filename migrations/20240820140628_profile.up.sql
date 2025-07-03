CREATE TABLE IF NOT EXISTS "profile_role" (
  "id" serial NOT NULL,
  "role_id" uuid NOT NULL,
  "name" varchar(128) NOT NULL,
  "type" smallint NOT NULL DEFAULT 0,
  "mode" smallint NOT NULL DEFAULT 0,
  PRIMARY KEY ("id"),
  UNIQUE ("role_id","name"),
  FOREIGN KEY ("role_id")
    REFERENCES "role" ("role_id") ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS "profile_user" (
  "id" serial NOT NULL,
  "user_id" uuid NOT NULL,
  "name" varchar(128) NOT NULL,
  "order" smallint NOT NULL DEFAULT 0,
  "value" bytea NOT NULL,
  "type" smallint NOT NULL DEFAULT 0,
  PRIMARY KEY ("id"),
  UNIQUE ("user_id","name","order"),
  FOREIGN KEY ("user_id")
    REFERENCES "user" ("user_id") ON UPDATE CASCADE ON DELETE CASCADE
);
