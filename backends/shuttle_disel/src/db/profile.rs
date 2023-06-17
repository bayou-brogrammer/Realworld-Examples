use actix::prelude::*;
use diesel::prelude::*;

use crate::api::profile::{FollowProfile, GetProfile, ProfileResponse, UnFollowProfile};
use crate::db::DbExecutor;
use crate::error::{AppError, AppResult};
use crate::models::follower::{Follower, NewFollower};
use crate::models::user::User;

impl Handler<GetProfile> for DbExecutor {
    type Result = AppResult<ProfileResponse>;

    fn handle(&mut self, msg: GetProfile, _: &mut Self::Context) -> Self::Result {
        use crate::schema::followers::dsl::*;
        use crate::schema::users::dsl::*;

        let mut conn = self.0.get()?;
        let user: User = {
            match users
                .filter(username.eq(msg.username))
                .first(&mut conn)
                .optional()?
            {
                Some(user) => user,
                None => return Err(crate::error::AppError::not_found("User Profile not found")),
            }
        };

        let following = match msg.auth {
            Some(auth) => followers
                .filter(user_id.eq(user.id))
                .filter(follower_id.eq(auth.user.id))
                .first::<Follower>(&mut conn)
                .optional()?
                .is_some(),
            None => false,
        };

        Ok(ProfileResponse::new(user, following))
    }
}

impl Handler<FollowProfile> for DbExecutor {
    type Result = AppResult<ProfileResponse>;

    fn handle(&mut self, msg: FollowProfile, _: &mut Self::Context) -> Self::Result {
        use crate::schema::followers::dsl::*;
        use crate::schema::users::dsl::*;

        let mut conn = self.0.get()?;

        let user: User = {
            match users
                .filter(username.eq(msg.username))
                .first(&mut conn)
                .optional()?
            {
                Some(user) => user,
                None => return Err(AppError::not_found("User Profile not found")),
            }
        };

        let followee: User = msg.auth.user;
        if user.id == followee.id {
            return Err(AppError::UnprocessableEntity(
                serde_json::json!({"error": "You cannot follow yourself"}),
            ));
        }

        match diesel::insert_into(followers)
            .values(&NewFollower {
                user_id: user.id,
                follower_id: followee.id,
            })
            .execute(&mut conn)
        {
            Ok(_) => Ok(ProfileResponse::new(user, true)),
            Err(_) => Err(AppError::UnprocessableEntity(
                serde_json::json!({"error": "You are already following this user"}),
            )),
        }
    }
}

impl Handler<UnFollowProfile> for DbExecutor {
    type Result = AppResult<ProfileResponse>;

    fn handle(&mut self, msg: UnFollowProfile, _: &mut Self::Context) -> Self::Result {
        use crate::schema::followers::dsl::*;
        use crate::schema::users::dsl::*;

        let mut conn = self.0.get()?;

        let user: User = {
            match users
                .filter(username.eq(msg.username))
                .first(&mut conn)
                .optional()?
            {
                Some(user) => user,
                None => return Err(AppError::not_found("User Profile not found")),
            }
        };

        let followee: User = msg.auth.user;
        if user.id == followee.id {
            return Err(AppError::UnprocessableEntity(
                serde_json::json!({"error": "You cannot unfollow yourself"}),
            ));
        }

        diesel::delete(followers)
            .filter(user_id.eq(user.id))
            .filter(follower_id.eq(followee.id))
            .returning(Follower::as_returning())
            .execute(&mut conn)?;

        Ok(ProfileResponse::new(user, false))
    }
}
