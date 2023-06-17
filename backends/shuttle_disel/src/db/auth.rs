use actix::prelude::*;
use diesel::prelude::*;

use crate::db::DbExecutor;
use crate::error::{AppError, AppResult};
use crate::models::user::User;
use crate::utils::auth::Auth;
use crate::utils::DecodeJwt;

// message handler implementations ↓

#[derive(Debug, Clone, Message)]
#[rtype(result = "AppResult<Auth>")]
pub struct GenerateAuth {
    pub token: String,
}

impl Handler<GenerateAuth> for DbExecutor {
    type Result = AppResult<Auth>;

    fn handle(&mut self, msg: GenerateAuth, _: &mut Self::Context) -> Self::Result {
        use crate::schema::users::dsl::*;

        let claims = msg.token.decode_jwt()?.claims;
        let mut conn = self.0.get()?;

        match users.find(claims.id).first::<User>(&mut conn) {
            Err(_) => Err(AppError::Unauthorized("Invalid Token")),
            Ok(user) => Ok(Auth {
                user,
                token: msg.token,
            }),
        }
    }
}
