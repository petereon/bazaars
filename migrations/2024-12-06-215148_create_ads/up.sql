-- Your SQL goes here
-- pub struct Ad {
--     pub id: i32,
--     pub title: String,
--     pub description: String,
--     pub price: f64,
--     pub status: String,
--     pub user_email: String,
--     pub user_phone: String,

--     pub created_at: chrono::NaiveDateTime,
--     pub updated_at: chrono::NaiveDateTime,
--     pub top_ad: bool,
-- }
CREATE TABLE ads (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    price DECIMAL(10,2) NOT NULL,
    status VARCHAR(50) NOT NULL,
    user_email VARCHAR(255) NOT NULL,
    user_phone VARCHAR(50) NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    top_ad BOOLEAN NOT NULL DEFAULT FALSE,
    images JSONB NOT NULL DEFAULT '[]'
);

CREATE INDEX idx_ads_price ON ads(price);
CREATE INDEX idx_ads_status ON ads(status);
CREATE INDEX idx_ads_updated_at ON ads(updated_at);
CREATE INDEX idx_ads_top_ad ON ads(top_ad);