use std::sync::Arc;

use anyhow::Error;
use axum::async_trait;
use bigdecimal::{BigDecimal, FromPrimitive};
use diesel::pg::Pg;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_query;
use diesel::PgConnection;
use diesel::QueryableByName;
use diesel::{debug_query, prelude::*};

use crate::db::schema::ads;
use crate::db::DbManager;
use crate::models::ad::{Ad, AdContent};

pub struct Cursor {
    pub cursor_name: String,
    pub pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

impl Cursor {
    pub fn new(cursor_name: String, db_manager: DbManager) -> Cursor {
        let pool = db_manager.get_read_pool();
        Cursor { cursor_name, pool }
    }

    pub fn get_next<T>(&self, count: u8) -> Result<Vec<T>, Error>
    where
        T: QueryableByName<Pg> + 'static, // Ensure T can be converted from SQL and has a 'static lifetime
    {
        let query = format!("FETCH FORWARD {} FROM {}", count, self.cursor_name);
        let conn = &mut self.pool.get().map_err(|e| Error::msg(e.to_string()))?;
        sql_query(query).load::<T>(conn).map_err(Error::from)
    }
}

#[derive(serde::Deserialize, Default, Clone)]
pub struct AdFilter {
    pub title_contains: Option<String>,
    pub description_contains: Option<String>,
    pub price_lt: Option<BigDecimal>,
    pub price_gt: Option<BigDecimal>,
    pub updated_at_lt: Option<chrono::NaiveDateTime>,
    pub updated_at_gt: Option<chrono::NaiveDateTime>,
}

#[async_trait]
pub trait AdRepo: Send + Sync {
    async fn new_cursor(&self, filter: AdFilter) -> Result<String, Error>;
    async fn fetch_from_cursor(&self, cursor_name: String, count: u8) -> Result<Vec<Ad>, Error>;
    async fn get_by_id(&self, id: i32) -> Result<Option<Ad>, Error>;
    async fn get_page(&self, page: u32, per_page: u32, filter: AdFilter) -> Result<Vec<Ad>, Error>;
    async fn create(&self, ad: AdContent, image_ids: Vec<String>) -> Result<Ad, Error>;
    async fn update(&self, id: i32, ad: Ad) -> Result<Ad, Error>;
    async fn delete(&self, id: i32) -> Result<usize, Error>;
}

#[derive(Clone)]
pub struct PostgresAdRepo {
    pub db_manager: DbManager,
}

impl PostgresAdRepo {
    pub fn new(db_manager: DbManager) -> Arc<PostgresAdRepo> {
        Arc::new(PostgresAdRepo { db_manager })
    }
}

#[async_trait]
impl AdRepo for PostgresAdRepo {
    async fn new_cursor(&self, filter: AdFilter) -> Result<String, Error> {
        let mut query = ads::table.into_boxed();

        if let Some(ref title_contains) = filter.title_contains {
            query = query.filter(ads::title.ilike(format!("%{}%", title_contains)));
        }

        if let Some(ref description_contains) = filter.description_contains {
            query = query.filter(ads::description.ilike(format!("%{}%", description_contains)));
        }

        if let Some(ref filter_price_lt) = filter.price_lt {
            query = query.filter(ads::price.lt(filter_price_lt));
        }

        if let Some(ref filter_price_gt) = filter.price_gt {
            query = query.filter(ads::price.gt(filter_price_gt));
        }

        if let Some(ref updated_at_lt) = filter.updated_at_lt {
            query = query.filter(ads::updated_at.lt(updated_at_lt));
        }

        if let Some(ref updated_at_gt) = filter.updated_at_gt {
            query = query.filter(ads::updated_at.gt(updated_at_gt));
        }

        let conn = &mut self
            .db_manager
            .get_write_pool()
            .get()
            .map_err(|e| Error::msg(e.to_string()))?;

        let cursor_name = format!(
            "c_{}",
            uuid::Uuid::new_v4().to_string().replace("-", "")[..10].to_string()
        );

        let cursor_query_str = format!(
            "DECLARE {} CURSOR WITH HOLD FOR {}",
            cursor_name,
            debug_query(&query).to_string()
        );

        println!("{}", cursor_query_str);

        let mut cursor_query = sql_query(cursor_query_str).into_boxed::<Pg>();

        if let Some(ref title_contains) = filter.title_contains {
            cursor_query =
                cursor_query.bind::<diesel::sql_types::Text, _>(format!("%{}%", title_contains));
        }

        if let Some(ref description_contains) = filter.description_contains {
            cursor_query = cursor_query
                .bind::<diesel::sql_types::Text, _>(format!("%{}%", description_contains));
        }

        if let Some(ref filter_price_lt) = filter.price_lt {
            cursor_query = cursor_query.bind::<diesel::sql_types::Numeric, _>(filter_price_lt);
        }

        if let Some(ref filter_price_gt) = filter.price_gt {
            cursor_query = cursor_query.bind::<diesel::sql_types::Numeric, _>(filter_price_gt);
        }

        if let Some(ref updated_at_lt) = filter.updated_at_lt {
            cursor_query = cursor_query.bind::<diesel::sql_types::Timestamp, _>(updated_at_lt);
        }

        if let Some(ref updated_at_gt) = filter.updated_at_gt {
            cursor_query = cursor_query.bind::<diesel::sql_types::Timestamp, _>(updated_at_gt);
        }

        println!("{}", debug_query(&cursor_query).to_string());

        cursor_query.execute(conn).map_err(Error::from)?;

        Ok(cursor_name)
    }

    async fn fetch_from_cursor(&self, cursor_name: String, count: u8) -> Result<Vec<Ad>, Error> {
        let query = format!("FETCH FORWARD {} FROM {}", count, cursor_name);
        let conn = &mut self
            .db_manager
            .get_read_pool()
            .get()
            .map_err(|e| Error::msg(e.to_string()))?;
        sql_query(query).load::<Ad>(conn).map_err(Error::from)
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<Ad>, Error> {
        ads::table
            .find(id)
            .first::<Ad>(
                &mut self
                    .db_manager
                    .get_read_pool()
                    .get()
                    .map_err(|e| Error::msg(e.to_string()))?,
            )
            .optional()
            .map_err(Error::from)
    }

    async fn get_page(
        &self,
        offset: u32,
        per_page: u32,
        filter: AdFilter,
    ) -> Result<Vec<Ad>, Error> {
        let mut query = ads::table.into_boxed();

        if let Some(ref title_contains) = filter.title_contains {
            query = query.filter(ads::title.ilike(format!("%{}%", title_contains)));
        }

        if let Some(ref description_contains) = filter.description_contains {
            query = query.filter(ads::description.ilike(format!("%{}%", description_contains)));
        }

        if let Some(ref filter_price_lt) = filter.price_lt {
            query = query.filter(ads::price.lt(filter_price_lt));
        }

        if let Some(ref filter_price_gt) = filter.price_gt {
            query = query.filter(ads::price.gt(filter_price_gt));
        }

        if let Some(ref updated_at_lt) = filter.updated_at_lt {
            query = query.filter(ads::updated_at.lt(updated_at_lt));
        }

        if let Some(ref updated_at_gt) = filter.updated_at_gt {
            query = query.filter(ads::updated_at.gt(updated_at_gt));
        }

        query = query.offset(offset.into()).limit(per_page.into());

        let conn = &mut self
            .db_manager
            .get_read_pool()
            .get()
            .map_err(|e| Error::msg(e.to_string()))?;
        let res = query.load::<Ad>(conn).map_err(Error::from)?;

        Ok(res)
    }

    async fn create(&self, ad: AdContent, image_ids: Vec<String>) -> Result<Ad, Error> {
        diesel::insert_into(ads::table)
            .values((
                ads::title.eq(ad.title),
                ads::description.eq(ad.description),
                ads::price.eq(BigDecimal::from_f64(ad.price).unwrap()),
                ads::status.eq("active"),
                ads::user_email.eq(ad.user_email),
                ads::user_phone.eq(ad.user_phone),
                ads::top_ad.eq(ad.top_ad),
                ads::images.eq(serde_json::to_value(image_ids).map_err(Error::from)?),
                ads::created_at.eq(chrono::Utc::now().naive_utc()),
                ads::updated_at.eq(chrono::Utc::now().naive_utc()),
            ))
            .get_result::<Ad>(
                &mut self
                    .db_manager
                    .get_write_pool()
                    .get()
                    .map_err(|e| Error::msg(e.to_string()))?,
            )
            .map_err(Error::from)
    }

    async fn update(&self, id: i32, ad: Ad) -> Result<Ad, Error> {
        diesel::update(ads::table.find(id))
            .set(&ad)
            .get_result::<Ad>(
                &mut self
                    .db_manager
                    .get_write_pool()
                    .get()
                    .map_err(|e| Error::msg(e.to_string()))?,
            )
            .map_err(Error::from)
    }

    async fn delete(&self, id: i32) -> Result<usize, Error> {
        diesel::delete(ads::table.find(id))
            .execute(
                &mut self
                    .db_manager
                    .get_write_pool()
                    .get()
                    .map_err(|e| Error::msg(e.to_string()))?,
            )
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        models::ad::AdContent,
        repos::ad_repo::{AdFilter, AdRepo, PostgresAdRepo},
    };
    use std::env;

    #[tokio::test]
    async fn test_ad_repo() {
        let db_manager = crate::db::DbManager::new(
            env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set")
                .as_str(),
        );

        let ad_repo = PostgresAdRepo::new(db_manager);

        for i in 0..10 {
            let ad = AdContent {
                title: "Test Ad".to_string(),
                description: "Test Description".to_string(),
                price: 100.into(),
                user_email: "test@test.com".to_string(),
                user_phone: "1234567890".to_string(),
                top_ad: false,
            };

            ad_repo
                .create(ad, vec![])
                .await
                .expect("Failed to create ad");
        }

        let cursor_name = ad_repo
            .new_cursor(AdFilter {
                title_contains: Some("test".to_string()),
                description_contains: None,
                price_lt: None,
                price_gt: None,
                updated_at_lt: None,
                updated_at_gt: None,
            })
            .await
            .expect("Failed to get cursor");

        let ads = ad_repo.fetch_from_cursor(cursor_name, 2).await;

        if ads.is_err() {
            println!("{:?}", ads);
        }

        assert_eq!(ads.is_ok(), true);

        println!("{:?}", ads);
    }
}
