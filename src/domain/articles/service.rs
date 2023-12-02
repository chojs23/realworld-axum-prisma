use axum::{extract::Path, Extension, Json};
use std::sync::Arc;

use crate::{
    app_error::AppError,
    domain::profiles::service::ProfilesService,
    extractor::{AuthUser, OptionalAuthUser},
    prisma::{self, article, article_tag, user, user_favorite_article, PrismaClient},
};

use super::{
    request::{ArticleCreateInput, ArticleUpdateInput},
    response::Article,
    ArticleBody,
};

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

    fn check_author(
        auth_user: &AuthUser,
        article: &prisma::article::Data,
    ) -> Result<bool, AppError> {
        if article.author_id == auth_user.user_id {
            Ok(true)
        } else {
            Err(AppError::BadRequest(String::from(
                "You are not the author of this article",
            )))
        }
    }

    async fn check_favorited(
        prisma: &Prisma,
        auth_user: &AuthUser,
        article_id: i32,
    ) -> Result<bool, AppError> {
        let data = prisma
            .user_favorite_article()
            .find_unique(
                user_favorite_article::UniqueWhereParam::UserIdArticleIdEquals(
                    auth_user.user_id,
                    article_id,
                ),
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
            .with(article::author::fetch())
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
            article: article.to_article(tag_list.unwrap_or_default(), false, false),
        }))
    }

    pub async fn update_article(
        auth_user: AuthUser,
        prisma: Prisma,
        Path(slug): Path<String>,
        Json(input): Json<ArticleBody<ArticleUpdateInput>>,
    ) -> Result<Json<ArticleBody<Article>>, AppError> {
        let ArticleBody {
            article:
                ArticleUpdateInput {
                    title,
                    description,
                    body,
                },
        } = input;

        let article = prisma
            .article()
            .find_unique(article::slug::equals(slug.clone()))
            .with(article::author::fetch())
            .exec()
            .await?
            .ok_or(AppError::NotFound(String::from("Article not found")))?;

        Self::check_author(&auth_user, &article)?;

        let updated_article = prisma
            .article()
            .update(
                article::slug::equals(slug.clone()),
                vec![
                    match &title {
                        Some(title) => article::slug::set(Self::slugify(title.as_str())),
                        None => article::slug::set(article.slug),
                    },
                    match title {
                        Some(title) => article::title::set(title),
                        None => article::title::set(article.title),
                    },
                    match description {
                        Some(description) => article::description::set(description),
                        None => article::description::set(article.description),
                    },
                    match body {
                        Some(body) => article::body::set(body),
                        None => article::body::set(article.body),
                    },
                ],
            )
            .with(article::author::fetch())
            .exec()
            .await?;

        Ok(Json::from(ArticleBody {
            article: updated_article.to_article(vec![], false, false),
        }))
    }

    pub async fn get_article(
        auth_user: OptionalAuthUser,
        prisma: Prisma,
        Path(slug): Path<String>,
    ) -> Result<Json<ArticleBody<Article>>, AppError> {
        let article = prisma
            .article()
            .find_unique(article::slug::equals(slug))
            .with(article::author::fetch())
            .exec()
            .await?
            .ok_or(AppError::NotFound(String::from("Article not found")))?;

        let tags = prisma
            .article_tag()
            .find_many(vec![article_tag::article_id::equals(article.id)])
            .exec()
            .await?
            .into_iter()
            .map(|tag| tag.tag)
            .collect::<Vec<String>>();

        if let Some(user) = auth_user.0 {
            let favorited = Self::check_favorited(&prisma, &user, article.id).await?;

            let followed =
                ProfilesService::check_following(&prisma, &user, article.author_id).await?;

            return Ok(Json::from(ArticleBody {
                article: article.to_article(tags, favorited, followed),
            }));
        }

        Ok(Json::from(ArticleBody {
            article: article.to_article(tags, false, false),
        }))
    }
}
