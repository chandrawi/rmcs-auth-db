CREATE TABLE "api" (
  "api_id" uuid NOT NULL,
  "name" varchar(64) NOT NULL,
  "address" varchar(128) NOT NULL,
  "category" varchar(64) NOT NULL,
  "description" varchar(255) NOT NULL,
  "password" varchar(128) NOT NULL,
  "access_key" bytea NOT NULL,
  PRIMARY KEY ("api_id"),
  UNIQUE ("name")
);

CREATE TABLE "api_procedure" (
  "procedure_id" uuid NOT NULL,
  "api_id" uuid NOT NULL,
  "name" varchar(64) NOT NULL,
  "description" varchar(255) NOT NULL,
  PRIMARY KEY ("procedure_id"),
  UNIQUE ("api_id","name"),
  FOREIGN KEY ("api_id")
    REFERENCES "api" ("api_id")
);
