use std::ops::DerefMut;

use actix::prelude::*;
use diesel::prelude::*;
use serde_json::json;
use slug::slugify;
use uuid::Uuid;

use crate::api::articles::{
    ArticleListResponse, ArticleResponse, ArticleResponseInner, CreateArticleOuter, DeleteArticle,
    FavoriteArticle, GetArticle, GetArticles, GetFeedArticles, UnFavoriteArticle,
    UpdateArticleOuter,
};
use crate::db::DbExecutor;
use crate::error::AppResult;
use crate::models::articles::{Article, NewArticle, NewFavoriteArticle, UpdateArticle};
use crate::models::tags::{ArticleTag, NewArticleTag};
use crate::models::user::User;

impl Handler<CreateArticleOuter> for DbExecutor {
    type Result = AppResult<ArticleResponse>;

    fn handle(&mut self, msg: CreateArticleOuter, _: &mut Self::Context) -> Self::Result {
        use crate::schema::articles;

        let author = msg.auth.user;

        // Generating the Uuid here since it will help make a unique slug
        // This is for when some articles may have similar titles such that they generate the same slug
        let slug = slugify(&msg.article.title);

        let new_article = NewArticle {
            slug,
            author_id: author.id,
            body: msg.article.body,
            title: msg.article.title,
            description: msg.article.description,
        };

        let mut conn = self.0.get()?;

        let article = match diesel::insert_into(articles::table)
            .values(&new_article)
            .get_result::<Article>(&mut conn)
        {
            Ok(article) => article,
            Err(_) => {
                return Err(crate::error::AppError::UnprocessableEntity(json!({
                    "error": format!("Article with slug '{}' already exists", new_article.slug)
                })))
            }
        };

        let _ = replace_tags(conn.deref_mut(), article.id, msg.article.tag_list)?;
        get_article_response(conn.deref_mut(), article.slug, Some(article.author_id))
    }
}

impl Handler<GetArticle> for DbExecutor {
    type Result = AppResult<ArticleResponse>;

    fn handle(&mut self, msg: GetArticle, _: &mut Self::Context) -> Self::Result {
        let mut conn = self.0.get()?;

        match msg.auth {
            Some(auth) => get_article_response(conn.deref_mut(), msg.slug, Some(auth.user.id)),
            None => get_article_response(conn.deref_mut(), msg.slug, None),
        }
    }
}

impl Handler<UpdateArticleOuter> for DbExecutor {
    type Result = AppResult<ArticleResponse>;

    fn handle(&mut self, msg: UpdateArticleOuter, _: &mut Self::Context) -> Self::Result {
        use crate::schema::articles;

        let mut conn = self.0.get()?;

        let article = articles::table
            .filter(articles::slug.eq(msg.slug))
            .get_result::<Article>(&mut conn)?;

        if msg.auth.user.id != article.author_id {
            return Err(crate::error::AppError::Unauthorized(
                "You are not authorized to update this article",
            ));
        }

        let slug = msg.article.title.as_ref().map(slugify);

        let updated = UpdateArticle {
            slug,
            body: msg.article.body,
            title: msg.article.title,
            description: msg.article.description,
        };

        let article = diesel::update(articles::table.find(article.id))
            .set(&updated)
            .get_result::<Article>(&mut conn)?;

        if let Some(updated_tags) = msg.article.tag_list {
            replace_tags(conn.deref_mut(), article.id, updated_tags)?;
        }

        get_article_response(conn.deref_mut(), article.slug, Some(article.author_id))
    }
}

impl Handler<DeleteArticle> for DbExecutor {
    type Result = AppResult<serde_json::Value>;

    fn handle(&mut self, msg: DeleteArticle, _: &mut Self::Context) -> Self::Result {
        use crate::schema::articles;

        let mut conn = self.0.get()?;

        let article = articles::table
            .filter(articles::slug.eq(msg.slug))
            .get_result::<Article>(&mut conn)?;

        if msg.auth.user.id != article.author_id {
            return Err(crate::error::AppError::Unauthorized(
                "You are not authorized to delete this article",
            ));
        }

        match diesel::delete(articles::table.filter(articles::id.eq(article.id))).execute(&mut conn)
        {
            Ok(_) => Ok(json!({ "message": "OK" })),
            Err(e) => Err(e.into()),
        }
    }
}

impl Handler<GetArticles> for DbExecutor {
    type Result = AppResult<ArticleListResponse>;

    fn handle(&mut self, msg: GetArticles, _: &mut Self::Context) -> Self::Result {
        use crate::schema::{articles, users};

        let mut conn = self.0.get()?;

        let mut query = articles::table.into_boxed();

        // Author username
        if let Some(ref author_name) = msg.params.author {
            let articles_by_author = articles::table
                .inner_join(users::table)
                .filter(users::username.eq(author_name))
                .select(articles::id)
                .load::<Uuid>(&mut conn)?;

            query = query.filter(articles::id.eq_any(articles_by_author));
        }

        // Favorited by user
        if let Some(favorited_username) = msg.params.favorited {
            let favorited_by_user = articles::table
                .inner_join(users::table)
                .filter(users::username.eq(favorited_username))
                .select(articles::id)
                .load::<Uuid>(&mut conn)?;

            query = query.filter(articles::id.eq_any(favorited_by_user));
        }

        // Tags
        if let Some(tag) = msg.params.tag {
            use crate::schema::article_tags;

            let tagged_article_ids: Vec<Uuid> = article_tags::table
                .filter(article_tags::tag_name.eq(tag))
                .select(article_tags::article_id)
                .load::<Uuid>(&mut conn)?;

            query = query.filter(articles::id.eq_any(tagged_article_ids));
        }

        let limit = std::cmp::min(msg.params.limit.unwrap_or(20), 100) as i64;
        let offset = msg.params.offset.unwrap_or(0) as i64;

        let articles = query
            .order(articles::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<Article>(&mut conn)?;

        match msg.auth {
            Some(auth) => get_article_list_response(conn.deref_mut(), articles, Some(auth.user.id)),
            None => get_article_list_response(conn.deref_mut(), articles, None),
        }
    }
}

impl Handler<GetFeedArticles> for DbExecutor {
    type Result = AppResult<ArticleListResponse>;

    fn handle(&mut self, msg: GetFeedArticles, _: &mut Self::Context) -> Self::Result {
        use crate::schema::{articles, followers};

        let mut conn = self.0.get()?;

        let limit = std::cmp::min(msg.params.limit.unwrap_or(20), 100) as i64;
        let offset = msg.params.offset.unwrap_or(0) as i64;
        let user_id = msg.auth.user.id;

        println!("user_id: {:?}", user_id);

        let following_ids = followers::table
            .filter(followers::follower_id.eq(user_id))
            .select(followers::user_id)
            .load::<Uuid>(&mut conn)?;

        println!("user_id: {:?}", user_id);

        let articles = articles::table
            .filter(articles::author_id.eq_any(following_ids))
            .order(articles::created_at.desc())
            .limit(limit)
            .offset(offset)
            .get_results::<Article>(&mut conn)?;

        get_article_list_response(conn.deref_mut(), articles, Some(user_id))
    }
}

impl Handler<FavoriteArticle> for DbExecutor {
    type Result = AppResult<ArticleResponse>;

    fn handle(&mut self, msg: FavoriteArticle, _: &mut Self::Context) -> Self::Result {
        use crate::schema::{articles, favorite_articles};

        let mut conn = self.0.get()?;

        let article = articles::table
            .filter(articles::slug.eq(msg.slug))
            .get_result::<Article>(&mut conn)?;

        match diesel::insert_into(favorite_articles::table)
            .values(NewFavoriteArticle {
                article_id: article.id,
                user_id: msg.auth.user.id,
            })
            .execute(&mut conn)
        {
            Ok(_) => get_article_response(conn.deref_mut(), article.slug, Some(article.author_id)),
            Err(_) => Err(crate::error::AppError::UnprocessableEntity(
                json!({ "errors": "Article already favorited" }),
            )),
        }
    }
}

impl Handler<UnFavoriteArticle> for DbExecutor {
    type Result = AppResult<ArticleResponse>;

    fn handle(&mut self, msg: UnFavoriteArticle, _: &mut Self::Context) -> Self::Result {
        use crate::schema::{articles, favorite_articles};

        let mut conn = self.0.get()?;

        let article = articles::table
            .filter(articles::slug.eq(msg.slug))
            .get_result::<Article>(&mut conn)?;

        diesel::delete(favorite_articles::table)
            .filter(favorite_articles::user_id.eq(msg.auth.user.id))
            .filter(favorite_articles::article_id.eq(article.id))
            .execute(&mut conn)?;

        get_article_response(conn.deref_mut(), article.slug, Some(article.author_id))
    }
}

// ================== HELPERS ================== //
fn get_article_response(
    conn: &mut PgConnection,
    slug: String,
    user_id: Option<Uuid>,
) -> AppResult<ArticleResponse> {
    use crate::schema::{articles, users};

    let (article, author) = articles::table
        .inner_join(users::table)
        .filter(articles::slug.eq(slug))
        .get_result::<(Article, User)>(conn)?;

    let (favorited, following) = match user_id {
        Some(user_id) => get_favorited_and_following(conn, article.id, author.id, user_id)?,
        None => (false, false),
    };

    let favorites_count = get_favorites_count(conn, article.id)?;
    let tags = select_tags_on_article(conn, article.id)?;

    Ok(ArticleResponse::new(
        article,
        author,
        tags,
        favorited,
        favorites_count,
        following,
    ))
}

fn get_article_list_response(
    conn: &mut PgConnection,
    articles: Vec<Article>,
    user_id: Option<Uuid>,
) -> AppResult<ArticleListResponse> {
    let article_list = articles
        .iter()
        .map(
            |article| match get_article_response(conn, article.slug.to_owned(), user_id) {
                Ok(response) => Ok(response.article),
                Err(e) => Err(e),
            },
        )
        .collect::<AppResult<Vec<ArticleResponseInner>>>()?;

    Ok(ArticleListResponse {
        articles_count: article_list.len(),
        articles: article_list,
    })
}

fn add_tag<T>(conn: &mut PgConnection, article_id: Uuid, tag_name: T) -> AppResult<ArticleTag>
where
    T: ToString,
{
    use crate::schema::article_tags;

    diesel::insert_into(article_tags::table)
        .values(NewArticleTag {
            article_id,
            tag_name: tag_name.to_string(),
        })
        .on_conflict((article_tags::article_id, article_tags::tag_name))
        .do_nothing()
        .get_result::<ArticleTag>(conn)
        .map_err(Into::into)
}

fn replace_tags<I>(conn: &mut PgConnection, article_id: Uuid, tags: I) -> AppResult<Vec<ArticleTag>>
where
    I: IntoIterator<Item = String>,
{
    delete_tags(article_id, conn)?;

    // this may look confusing but collect can convert to this
    // https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect
    tags.into_iter()
        .map(|tag_name| add_tag(conn, article_id, &tag_name))
        .collect::<AppResult<Vec<ArticleTag>>>()
}

fn delete_tags(article_id: Uuid, conn: &mut PgConnection) -> AppResult<usize> {
    use crate::schema::article_tags;

    diesel::delete(article_tags::table.filter(article_tags::article_id.eq(article_id)))
        .execute(conn)
        .map_err(Into::into)
}

fn get_favorited_and_following(
    conn: &mut PgConnection,
    article_id: Uuid,
    author_id: Uuid,
    user_id: Uuid,
) -> AppResult<(bool, bool)> {
    use crate::schema::{favorite_articles, followers, users};

    let (_, favorite_id, follow_id) = users::table
        .left_join(
            favorite_articles::table.on(favorite_articles::article_id
                .eq(users::id)
                .and(favorite_articles::article_id.eq(article_id))),
        )
        .left_join(
            followers::table.on(followers::follower_id
                .eq(users::id)
                .and(followers::user_id.eq(author_id))),
        )
        .filter(users::id.eq(user_id))
        .select((
            users::id,
            favorite_articles::user_id.nullable(),
            followers::user_id.nullable(),
        ))
        .get_result::<(Uuid, Option<Uuid>, Option<Uuid>)>(conn)?;

    Ok((favorite_id.is_some(), follow_id.is_some()))
}

fn get_favorites_count(conn: &mut PgConnection, article_id: Uuid) -> AppResult<i64> {
    use crate::schema::favorite_articles;

    Ok(favorite_articles::table
        .filter(favorite_articles::article_id.eq(article_id))
        .count()
        .get_result::<i64>(conn)?)
}

fn select_tags_on_article(conn: &mut PgConnection, article_id: Uuid) -> AppResult<Vec<String>> {
    use crate::schema::article_tags;

    Ok(article_tags::table
        .filter(article_tags::article_id.eq(article_id))
        .select(article_tags::tag_name)
        .load(conn)?)
}
