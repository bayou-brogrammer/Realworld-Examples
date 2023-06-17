use actix::prelude::*;
use diesel::prelude::*;

use crate::api::tags::{GetTags, TagsResponse};
use crate::db::DbExecutor;
use crate::error::AppResult;
use crate::models::tags::ArticleTag;

impl Handler<GetTags> for DbExecutor {
    type Result = AppResult<TagsResponse>;

    fn handle(&mut self, _: GetTags, _: &mut Self::Context) -> Self::Result {
        use crate::schema::article_tags::dsl::*;

        let mut conn = self.0.get()?;

        println!("Getting tags");

        let tags = article_tags
            .distinct_on(tag_name)
            .load::<ArticleTag>(&mut conn)?;

        let tag_list = tags
            .iter()
            .map(|tag| tag.tag_name.to_owned())
            .collect::<Vec<String>>();

        Ok(TagsResponse { tags: tag_list })
    }
}
