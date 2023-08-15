CREATE TABLE IF NOT EXISTS "api" (
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

CREATE TABLE IF NOT EXISTS "api_procedure" (
  "procedure_id" uuid NOT NULL,
  "api_id" uuid NOT NULL,
  "name" varchar(64) NOT NULL,
  "description" varchar(255) NOT NULL,
  PRIMARY KEY ("procedure_id"),
  UNIQUE ("api_id","name"),
  FOREIGN KEY ("api_id")
    REFERENCES "api" ("api_id")
);

INSERT INTO "api" ("api_id", "name", "address", "category", "description", "password", "access_key")
  VALUES (
    '00000000-0000-0000-0000-000000000000', 
    'default', 
    '127.0.0.1:9000', 
    'RESOURCE', 
    '', 
    '$argon2id$v=19$m=19456,t=2,p=1$RhlWFMLdNpLB/+schGqD3A$x2EzQ9thjKh6WSxsRLJtR1kOzpSPx/37Lka4bAl1uLQ', -- Ap1_P4s5w0rd
    decode('70edd2005f9d2c3bd9cf50616c96d14aab724436e5f47fe740cac446d952028c', 'hex')
  ) 
  ON CONFLICT ("api_id") DO NOTHING;
