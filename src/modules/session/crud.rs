use crate::modules::session::model::{Message, Session};
use bson::{doc, oid::ObjectId};
use mongodb::{Collection, Database};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

const COLLECTION_NAME: &str = "sessions";
const CACHE_TTL: u64 = 3600; // 1 hour

pub struct SessionCrud {
    collection: Collection<Session>,
    redis: ConnectionManager,
}

impl SessionCrud {
    pub fn new(db: &Database, redis: ConnectionManager) -> Self {
        Self {
            collection: db.collection(COLLECTION_NAME),
            redis,
        }
    }

    fn cache_key(id: &ObjectId) -> String {
        format!("session:{}", id.to_hex())
    }

    pub async fn create(&self, session: Session) -> Result<ObjectId, mongodb::error::Error> {
        let result = self.collection.insert_one(session).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn find_by_id(&self, id: &ObjectId) -> Result<Option<Session>, mongodb::error::Error> {
        // Try cache first
        let cache_key = Self::cache_key(id);
        let mut redis = self.redis.clone();

        if let Ok(cached) = redis.get::<_, String>(&cache_key).await {
            if let Ok(session) = serde_json::from_str::<Session>(&cached) {
                return Ok(Some(session));
            }
        }

        // Fallback to database
        let session = self.collection.find_one(doc! { "_id": id }).await?;

        // Cache the result
        if let Some(ref s) = session {
            if let Ok(json) = serde_json::to_string(s) {
                let _: Result<(), _> = redis.set_ex(&cache_key, json, CACHE_TTL).await;
            }
        }

        Ok(session)
    }

    pub async fn find_all(&self, limit: i64) -> Result<Vec<Session>, mongodb::error::Error> {
        use futures::TryStreamExt;

        let cursor = self
            .collection
            .find(doc! {})
            .sort(doc! { "updated_at": -1 })
            .limit(limit)
            .await?;

        cursor.try_collect().await
    }

    pub async fn count(&self) -> Result<u64, mongodb::error::Error> {
        self.collection.count_documents(doc! {}).await
    }

    pub async fn add_message(&self, id: &ObjectId, message: Message) -> Result<bool, mongodb::error::Error> {
        let result = self
            .collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$push": { "messages": bson::to_bson(&message).unwrap() },
                    "$set": { "updated_at": bson::DateTime::now() }
                },
            )
            .await?;

        // Invalidate cache
        let cache_key = Self::cache_key(id);
        let mut redis = self.redis.clone();
        let _: Result<(), _> = redis.del(&cache_key).await;

        Ok(result.modified_count > 0)
    }

    pub async fn delete(&self, id: &ObjectId) -> Result<bool, mongodb::error::Error> {
        let result = self.collection.delete_one(doc! { "_id": id }).await?;

        // Invalidate cache
        let cache_key = Self::cache_key(id);
        let mut redis = self.redis.clone();
        let _: Result<(), _> = redis.del(&cache_key).await;

        Ok(result.deleted_count > 0)
    }

    pub async fn update_title(&self, id: &ObjectId, title: String) -> Result<bool, mongodb::error::Error> {
        let result = self
            .collection
            .update_one(
                doc! { "_id": id },
                doc! {
                    "$set": {
                        "title": title,
                        "updated_at": bson::DateTime::now()
                    }
                },
            )
            .await?;

        // Invalidate cache
        let cache_key = Self::cache_key(id);
        let mut redis = self.redis.clone();
        let _: Result<(), _> = redis.del(&cache_key).await;

        Ok(result.modified_count > 0)
    }
}
