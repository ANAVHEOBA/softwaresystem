use crate::modules::ai::model::AiCompletion;
use bson::{doc, oid::ObjectId};
use mongodb::{Collection, Database};

const COLLECTION_NAME: &str = "ai_completions";

pub struct AiCrud {
    collection: Collection<AiCompletion>,
}

impl AiCrud {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(COLLECTION_NAME),
        }
    }

    pub async fn create(&self, completion: AiCompletion) -> Result<ObjectId, mongodb::error::Error> {
        let result = self.collection.insert_one(completion).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn find_by_id(&self, id: &ObjectId) -> Result<Option<AiCompletion>, mongodb::error::Error> {
        self.collection.find_one(doc! { "_id": id }).await
    }

    pub async fn find_recent(&self, limit: i64) -> Result<Vec<AiCompletion>, mongodb::error::Error> {
        use futures::TryStreamExt;

        let cursor = self
            .collection
            .find(doc! {})
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .await?;

        cursor.try_collect().await
    }

    pub async fn count(&self) -> Result<u64, mongodb::error::Error> {
        self.collection.count_documents(doc! {}).await
    }
}
