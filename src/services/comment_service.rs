use chrono::Utc;
use futures::stream::StreamExt;
use mongodb::{
    Collection,
    bson::{Document, doc, oid::ObjectId, from_document},
};
use std::sync::Arc;
use validator::Validate;

use crate::{
    db::{ DatabaseState},
    errors::AppError,
    models::{
        comment_model::{Comment, CommentData, CreateCommentSchema},
        task_model::Task,
        
    },
    services::permission_service::PermissionService,
};

pub struct CommentService {
    db: Arc<DatabaseState>,
}

impl CommentService {
    pub fn new(db_state: Arc<DatabaseState>) -> Self {
        Self { db: db_state }
    }

    fn comments_collection(&self) -> Collection<Comment> {
        self.db.db.collection("comments")
    }

    async fn check_permissions(
        &self,
        task_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<(), AppError> {
        let task = self
            .db
            .db
            .collection::<Task>("tasks")
            .find_one(doc! {"_id": task_id})
            .await
            .map_err(|_| AppError::DatabaseError("Failed to fetch task".to_string()))?
            .ok_or(AppError::NotFound("Task not found".to_string()))?;

        PermissionService::new(&self.db.db)
            .can_access_project(task.project_id, user_id)
            .await?;

        Ok(())
    }    pub async fn create_comment(
        &self,
        task_id: ObjectId,
        author_id: ObjectId,
        schema: CreateCommentSchema,
    ) -> Result<CommentData, AppError> {
        schema
            .validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        self.check_permissions(task_id, author_id).await?;

        let new_comment = Comment {
            id: None,
            task_id,
            user_id: author_id,
            content: schema.content,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = self.comments_collection().insert_one(&new_comment).await
            .map_err(|e| AppError::DatabaseError(format!("Failed to insert comment: {}", e)))?;
        let comment_id = result.inserted_id.as_object_id().unwrap();
        let comments = self
            .get_comments_for_task(task_id, author_id, Some(comment_id))
            .await?;
        comments
            .into_iter()
            .next()
            .ok_or(AppError::InternalServerError)
    }    pub async fn get_comments_for_task(
        &self,
        task_id: ObjectId,
        user_id: ObjectId,
        single_comment_id: Option<ObjectId>
    )-> Result<Vec<CommentData>, AppError> {

        self.check_permissions(task_id, user_id).await?;

        // //? Pipeline de agregación para obtener comentarios con datos del usuario
        let mut initial_match = doc! {"task_id": task_id};
        if let Some(comment_id) = single_comment_id {
            initial_match.insert("_id", comment_id);
        }

        let pipeline: Vec<Document> = vec![
            doc!{ "$match": initial_match },
            doc!{ "$sort": { "created_at": 1 } },
            doc!{
                "$lookup": {
                    "from": "users",
                    "localField": "user_id",
                    "foreignField": "_id",
                    "as": "author"
                }
            },

            doc!{"$unwind": "$author"},
            doc!{
                "$project": {
                    "_id": 0,
                    "id": {"$toString": "$_id"},
                    "task_id": {"$toString": "$task_id"},
                    "content": 1,
                    "created_at": 1,
                    "updated_at": 1,
                    "author": {
                        "id": {"$toString": "$author._id"},
                        "username": "$author.username",
                        "email": "$author.email",
                    }
                }
            }
        ];

        let mut cursor = self.comments_collection().aggregate(pipeline).await
            .map_err(|e| AppError::DatabaseError(format!("Failed to fetch comments: {}", e)))?;
        let mut comments = Vec::new();

        while let Some(doc) = cursor.next().await {
            let doc = doc.map_err(|e| AppError::DatabaseError(format!("Failed to fetch document: {}", e)))?;
            let comment_data: CommentData = from_document(doc)
                .map_err(|e| AppError::DatabaseError(format!("Failed to deserialize comment: {}", e)))?;
            comments.push(comment_data);
        }

        Ok(comments)
    }
}
