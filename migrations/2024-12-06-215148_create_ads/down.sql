-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS idx_ads_price;
DROP INDEX IF EXISTS idx_ads_status;
DROP INDEX IF EXISTS idx_ads_updated_at;
DROP INDEX IF EXISTS idx_ads_top_ad;
DROP TABLE IF EXISTS ads;