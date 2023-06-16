use actix::prelude::*;
use diesel::prelude::*;

use crate::api::profile::{GetProfile, ProfileResponse};
use crate::db::DbExecutor;
use crate::error::AppResult;
use crate::models::follower::Follower;
use crate::models::user::User;

impl Handler<GetProfile> for DbExecutor {
    type Result = AppResult<ProfileResponse>;

    fn handle(&mut self, msg: GetProfile, _: &mut Self::Context) -> Self::Result {
        use crate::schema::followers::dsl::*;
        use crate::schema::users::dsl::*;

        let mut conn = self.0.get().unwrap();
        let user: User = {
            match users
                .filter(username.eq(msg.username))
                .first(&mut conn)
                .optional()?
            {
                Some(user) => user,
                None => return Err(crate::error::AppError::not_found("Profile not found")),
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
