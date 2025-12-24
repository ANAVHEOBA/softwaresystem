use crate::modules::stt::model::SttTranscription;
use bson::{doc, oid::ObjectId};
use mongodb::{Collection, Database};

const COLLECTION_NAME: &str = "stt_transcriptions";

pub struct SttCrud {
    collection: Collection<SttTranscription>,
}

impl SttCrud {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection(COLLECTION_NAME),
        }
    }

    pub async fn create(&self, transcription: SttTranscription) -> Result<ObjectId, mongodb::error::Error> {
        let result = self.collection.insert_one(transcription).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn find_by_id(&self, id: &ObjectId) -> Result<Option<SttTranscription>, mongodb::error::Error> {
        self.collection.find_one(doc! { "_id": id }).await
    }

    pub async fn find_all(&self, limit: i64) -> Result<Vec<SttTranscription>, mongodb::error::Error> {
        use futures::TryStreamExt;

        let cursor = self
            .collection
            .find(doc! {})
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .await?;

        cursor.try_collect().await
    }

    pub async fn find_by_session(&self, session_id: &str, limit: i64) -> Result<Vec<SttTranscription>, mongodb::error::Error> {
        use futures::TryStreamExt;

        let cursor = self
            .collection
            .find(doc! { "session_id": session_id })
            .sort(doc! { "created_at": -1 })
            .limit(limit)
            .await?;

        cursor.try_collect().await
    }

    pub async fn count(&self) -> Result<u64, mongodb::error::Error> {
        self.collection.count_documents(doc! {}).await
    }

    pub async fn update_ai_response(&self, id: &ObjectId, ai_response: String) -> Result<bool, mongodb::error::Error> {
        let result = self
            .collection
            .update_one(
                doc! { "_id": id },
                doc! { "$set": { "ai_response": ai_response } },
            )
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn delete(&self, id: &ObjectId) -> Result<bool, mongodb::error::Error> {
        let result = self.collection.delete_one(doc! { "_id": id }).await?;
        Ok(result.deleted_count > 0)
    }
}
