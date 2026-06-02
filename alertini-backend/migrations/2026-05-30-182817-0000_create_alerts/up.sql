-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    car_id UUID NOT NULL REFERENCES vehicles(id),
    note TEXT NOT NULL,
    reporter_id UUID NOT NULL REFERENCES users(id)
);
