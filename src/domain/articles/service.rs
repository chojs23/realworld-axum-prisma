use axum::{extract::State, Extension, Json};
use std::sync::Arc;

use crate::{
    app_error::AppError,
    config::AppContext,
    extractor::AuthUser,
    prisma::{article_tag, user, user_favorite_article, PrismaClient},
};

use super::{request::ArticleCreateInput, response::Article, ArticleBody};

type Prisma = Extension<Arc<PrismaClient>>;

pub struct ArticlesService;

impl ArticlesService {
    fn slugify(title: &str) -> String {
        title
            .chars()
            .map(|c| match c {
                ' ' => '-',
                _ => c,
            })
            .collect::<String>()
            .to_lowercase()
    }

    async fn check_favorited(
        prisma: &Prisma,
        user_id: i32,
        article_id: i32,
    ) -> Result<bool, AppError> {
        let data = prisma
            .user_favorite_article()
            .find_unique(
                user_favorite_article::UniqueWhereParam::UserIdArticleIdEquals(user_id, article_id),
            )
            .exec()
            .await?;

        Ok(data.is_some())
    }

    pub async fn create_article(
        auth_user: AuthUser,
        prisma: Prisma,
        Json(input): Json<ArticleBody<ArticleCreateInput>>,
    ) -> Result<Json<ArticleBody<Article>>, AppError> {
        let ArticleBody {
            article:
                ArticleCreateInput {
                    title,
                    description,
                    body,
                    tag_list,
                },
        } = input;

        let author = prisma
            .user()
            .find_unique(user::id::equals(auth_user.user_id))
            .exec()
            .await?
            .ok_or(AppError::NotFound(String::from("User not found")))?;

        let article = prisma
            .article()
            .create(
                Self::slugify(title.as_str()),
                title,
                description,
                body,
                user::id::equals(auth_user.user_id),
                vec![],
            )
            .exec()
            .await?;

        if let Some(tag_list) = tag_list.clone() {
            let _ = prisma
                .article_tag()
                .create_many(
                    tag_list
                        .iter()
                        .map(|tag| {
                            article_tag::create_unchecked(tag.to_string(), article.id, vec![])
                        })
                        .collect(),
                )
                .exec()
                .await?;
        }

        Ok(Json::from(ArticleBody {
            article: article.to_article(tag_list.unwrap_or_default(), false, 0, author.into()),
        }))
    }
}
