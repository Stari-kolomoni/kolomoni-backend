use chrono::Utc;
use kolomoni_auth::ArgonHasher;
use kolomoni_core::id::UserId;
use sqlx::PgConnection;
use uuid::Uuid;

use super::UserQueryResult;
use crate::{IntoExternalModel, IntoInternalModel, QueryResult};


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UserRegistrationInfo {
    pub username: String,
    pub display_name: String,
    pub password: String,
}


pub struct UserMutation;

impl UserMutation {
    pub async fn create_user(
        database_connection: &mut PgConnection,
        hasher: &ArgonHasher,
        user_registration_info: UserRegistrationInfo,
    ) -> UserQueryResult<super::UserModel> {
        let hashed_password = hasher.hash_password(&user_registration_info.password)?;

        let user_uuid = Uuid::now_v7();
        let registration_time = Utc::now();


        let user_model = super::UserModel {
            id: UserId::new(user_uuid),
            username: user_registration_info.username,
            display_name: user_registration_info.display_name,
            hashed_password: hashed_password.to_string(),
            joined_at: registration_time,
            last_active_at: registration_time,
            last_modified_at: registration_time,
        }
        .into_internal_model();


        let newly_created_user = sqlx::query_as!(
            super::InternalUserModel,
            "INSERT INTO kolomoni.user \
                (id, username, display_name, hashed_password, \
                joined_at, last_active_at, last_modified_at) \
            VALUES ($1, $2, $3, $4, $5, $6, $7) \
            RETURNING \
                id, username, display_name, hashed_password, \
                joined_at, last_active_at, last_modified_at",
            user_model.id,
            user_model.username,
            user_model.display_name,
            user_model.hashed_password,
            user_model.joined_at,
            user_model.last_active_at,
            user_model.last_modified_at,
        )
        .fetch_one(database_connection)
        .await?;

        Ok(newly_created_user.into_external_model())
    }

    pub async fn change_display_name_by_user_id(
        database_connection: &mut PgConnection,
        user_id: UserId,
        new_display_name: &str,
    ) -> QueryResult<super::UserModel> {
        let updated_user_model = sqlx::query_as!(
            super::InternalUserModel,
            "UPDATE kolomoni.user \
            SET \
                display_name = $1, \
                last_modified_at = $2 \
            WHERE id = $3 \
            RETURNING \
                id, username, display_name, hashed_password, \
                joined_at, last_active_at, last_modified_at",
            new_display_name,
            Utc::now(),
            user_id.into_uuid()
        )
        .fetch_one(database_connection)
        .await?;

        Ok(updated_user_model.into_external_model())
    }
}
