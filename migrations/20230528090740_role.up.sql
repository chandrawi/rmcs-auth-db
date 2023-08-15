CREATE TABLE IF NOT EXISTS "role" (
  "role_id" uuid NOT NULL,
  "api_id" uuid NOT NULL,
  "name" varchar(64) NOT NULL,
  "multi" boolean NOT NULL DEFAULT true,
  "ip_lock" boolean NOT NULL DEFAULT false,
  "access_duration" int NOT NULL,
  "refresh_duration" int NOT NULL,
  PRIMARY KEY ("role_id"),
  UNIQUE ("api_id","name"),
  FOREIGN KEY ("api_id")
    REFERENCES "api" ("api_id")
);

CREATE TABLE IF NOT EXISTS "role_access" (
  "role_id" uuid NOT NULL,
  "procedure_id" uuid NOT NULL,
  PRIMARY KEY ("role_id","procedure_id"),
  FOREIGN KEY ("role_id")
    REFERENCES "role" ("role_id"),
  FOREIGN KEY ("procedure_id")
    REFERENCES "api_procedure" ("procedure_id")
);
