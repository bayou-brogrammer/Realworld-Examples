use actix::prelude::*;
use diesel::prelude::*;
use libreauth::pass::HashBuilder;

use crate::{
    api::user::{LoginUser, RegistrationUser, UpdateUserOuter, UserResponse},
    error::{AppError, AppResult},
    models::user::{NewUser, User, UserChange},
    utils::HASHER,
};

use super::DbExecutor;

impl Handler<RegistrationUser> for DbExecutor {
    type Result = AppResult<UserResponse>;

    fn handle(&mut self, msg: RegistrationUser, _: &mut Self::Context) -> Self::Result {
        use crate::schema::users::dsl::*;

        let new_user = NewUser {
            bio: None,
            image: None,
            email: msg.email.clone(),
            username: msg.username.clone(),
            password: HASHER.hash(&msg.password)?,
        };

        let mut conn = self.0.get()?;

        match diesel::insert_into(users)
            .values(new_user)
            .get_result::<User>(&mut conn)
        {
            Ok(user) => Ok(user.into()),
            Err(e) => Err(e.into()),
        }
    }
}

impl Handler<LoginUser> for DbExecutor {
    type Result = AppResult<UserResponse>;

    fn handle(&mut self, msg: LoginUser, _: &mut Self::Context) -> Self::Result {
        use crate::schema::users::dsl::*;
        let mut conn = self.0.get()?;

        let stored_user: User = users.filter(email.eq(msg.email)).first(&mut conn)?;
        let checker = HashBuilder::from_phc(&stored_user.password)?;

        let provided_password_raw = &msg.password;
        if checker.is_valid(provided_password_raw) {
            if checker.needs_update(Some(crate::utils::PWD_SCHEME_VERSION)) {
                let new_password = HASHER.hash(provided_password_raw)?;
                return match diesel::update(users.find(stored_user.id))
                    .set(password.eq(new_password))
                    .get_result::<User>(&mut conn)
                {
                    Ok(user) => Ok(user.into()),
                    Err(e) => Err(e.into()),
                };
            }
            Ok(stored_user.into())
        } else {
            Err(AppError::Unauthorized("Wrong password"))
        }
    }
}

impl Handler<UpdateUserOuter> for DbExecutor {
    type Result = AppResult<UserResponse>;

    fn handle(&mut self, msg: UpdateUserOuter, _: &mut Self::Context) -> Self::Result {
        use crate::schema::users::dsl::*;

        let auth = msg.auth;
        let update_user = msg.update_user;

        let updated_password = match update_user.password {
            Some(updated_password) => Some(HASHER.hash(&updated_password)?),
            None => None,
        };

        let updated_user = UserChange {
            bio: update_user.bio,
            email: update_user.email,
            image: update_user.image,
            password: updated_password,
            username: update_user.username,
        };

        let mut conn = self.0.get()?;
        let current_user = users.find(auth.user.id).first::<User>(&mut conn)?;
        match diesel::update(&current_user)
            .set(updated_user)
            .get_result::<User>(&mut conn)
        {
            Err(e) => Err(e.into()),
            Ok(user) => Ok(user.into()),
        }
    }
}
