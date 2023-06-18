use std::ops::DerefMut;

use actix::prelude::*;
use diesel::prelude::*;
use serde_json::json;
use uuid::Uuid;

use crate::api::comments::{
    AddCommentOuter, CommentListResponse, CommentResponse, CommentResponseInner, DeleteComment,
    GetComments,
};
use crate::api::profile::ProfileResponseInner;
use crate::db::DbExecutor;
use crate::error::AppResult;
use crate::models::comment::{Comment, NewComment};
use crate::models::follower::Follower;
use crate::models::user::User;
use crate::utils::CustomDateTime;

impl Handler<AddCommentOuter> for DbExecutor {
    type Result = AppResult<CommentResponse>;

    fn handle(&mut self, msg: AddCommentOuter, _: &mut Self::Context) -> Self::Result {
        use crate::schema::{articles, comments};

        let mut conn = self.0.get()?;

        let article_id = articles::table
            .filter(articles::slug.eq(msg.slug))
            .select(articles::id)
            .get_result::<Uuid>(&mut conn)?;

        let user_id = msg.auth.user.id;

        let new_comment = NewComment {
            user_id,
            article_id,
            body: msg.comment.body,
        };

        let comment = diesel::insert_into(comments::table)
            .values(new_comment)
            .get_result::<Comment>(&mut conn)?;

        get_comment_response(conn.deref_mut(), comment.id, Some(user_id))
    }
}

impl Handler<GetComments> for DbExecutor {
    type Result = AppResult<CommentListResponse>;

    fn handle(&mut self, msg: GetComments, _: &mut Self::Context) -> Self::Result {
        use crate::schema::{articles, comments};

        let mut conn = self.0.get()?;

        let article_id = articles::table
            .filter(articles::slug.eq(msg.slug))
            .select(articles::id)
            .get_result::<Uuid>(&mut conn)?;

        let comments = comments::table
            .filter(comments::article_id.eq(article_id))
            .load::<Comment>(&mut conn)?;

        match msg.auth {
            Some(auth) => get_comment_list_response(conn.deref_mut(), comments, Some(auth.user.id)),
            None => get_comment_list_response(conn.deref_mut(), comments, None),
        }
    }
}

impl Handler<DeleteComment> for DbExecutor {
    type Result = AppResult<serde_json::Value>;

    fn handle(&mut self, msg: DeleteComment, _: &mut Self::Context) -> Self::Result {
        use crate::schema::comments;

        let mut conn = self.0.get()?;

        let comment = comments::table
            .filter(comments::id.eq(msg.comment_id))
            .get_result::<Comment>(&mut conn)?;

        if msg.auth.user.id != comment.user_id {
            return Err(crate::error::AppError::Unauthorized(
                "You are not authorized to delete this comment",
            ));
        }

        match diesel::delete(comments::table.filter(comments::id.eq(msg.comment_id)))
            .execute(&mut conn)
        {
            Ok(_) => Ok(json!({ "message": "OK" })),
            Err(e) => Err(e.into()),
        }
    }
}

fn get_comment_response(
    conn: &mut PgConnection,
    comment_id: i32,
    user_id: Option<Uuid>,
) -> AppResult<CommentResponse> {
    use crate::schema::{comments, followers, users};

    let (comment, commenter) = comments::table
        .inner_join(users::table)
        .filter(comments::id.eq(comment_id))
        .get_result::<(Comment, User)>(conn)?;

    let following = match user_id {
        Some(user_id) => followers::table
            .filter(followers::user_id.eq(user_id))
            .filter(followers::follower_id.eq(commenter.id))
            .first::<Follower>(conn)
            .optional()?
            .is_some(),
        None => false,
    };

    Ok(CommentResponse {
        comment: CommentResponseInner {
            id: comment.id,
            body: comment.body,
            created_at: CustomDateTime(comment.created_at),
            updated_at: CustomDateTime(comment.updated_at),
            author: ProfileResponseInner {
                following,
                bio: commenter.bio,
                image: commenter.image,
                username: commenter.username,
            },
        },
    })
}

fn get_comment_list_response(
    conn: &mut PgConnection,
    comments: Vec<Comment>,
    user_id: Option<Uuid>,
) -> AppResult<CommentListResponse> {
    let comment_list = comments
        .iter()
        .map(
            |comment| match get_comment_response(conn, comment.id.to_owned(), user_id) {
                Ok(response) => Ok(response.comment),
                Err(e) => Err(e),
            },
        )
        .collect::<AppResult<Vec<CommentResponseInner>>>()?;

    Ok(CommentListResponse {
        comments: comment_list,
    })
}
