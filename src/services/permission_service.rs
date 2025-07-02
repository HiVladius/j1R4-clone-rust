use crate::{errors::AppError, models::project_models::Project};
use mongodb::{
    Collection, Database,
    bson::{doc, oid::ObjectId},
};

pub struct PermissionService<'a> {
    db: &'a Database,
}

impl<'a> PermissionService<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    fn projects_collection(&self) -> Collection<Project> {
        self.db.collection("projects")
    }
    pub async fn can_access_project(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<Project, AppError> {
        let project = self
            .projects_collection()
            .find_one(doc! {"_id": project_id})
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        if project.owner_id != user_id && !project.members.contains(&user_id) {
            return Err(AppError::Unauthorized(
                "You do not have access to this project".to_string(),
            ));
        }

        Ok(project)
    }

    pub async fn is_project_owner(
        &self,
        project_id: ObjectId,
        user_id: ObjectId,
    ) -> Result<Project, AppError> {
        let project = self
            .projects_collection()
            .find_one(doc! {"_id": project_id})
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        if project.owner_id != user_id {
            return Err(AppError::Unauthorized(
                "You are not the owner of this project".to_string(),
            ));
        }
        Ok(project)
    }
}
