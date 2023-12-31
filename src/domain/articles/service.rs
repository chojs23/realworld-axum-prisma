use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use prisma_client_rust::chrono;
use prisma_client_rust::Direction;
use std::sync::Arc;

use crate::{
    app_error::AppError,
    domain::profiles::service::ProfilesService,
    extractor::{AuthUser, OptionalAuthUser},
    prisma::{
        self, article, article_tag, comment, user, user_favorite_article,
        user_follows::followed_by_id, PrismaClient,
    },
};

use super::{
    request::{ArticleCreateInput, ArticleListQuery, ArticleUpdateInput, CommentCreateInput},
    response::{Article, Comment},
    ArticleBody, ArticlesBody, CommentBody, CommentsBody,
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

    fn slug_hash(slug: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        slug.hash(&mut hasher);
        hasher.finish()
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
            article: article.to_article(false, false),
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
            article: updated_article.to_article(false, false),
        }))
    }

    pub async fn delete_article(
        auth_user: AuthUser,
        prisma: Prisma,
        Path(slug): Path<String>,
    ) -> Result<Json<String>, AppError> {
        let article = prisma
            .article()
            .find_unique(article::slug::equals(slug.clone()))
            .with(article::author::fetch())
            .exec()
            .await?
            .ok_or(AppError::NotFound(String::from("Article not found")))?;

        Self::check_author(&auth_user, &article)?;

        let _ = prisma
            .article()
            .update(
                article::slug::equals(slug.clone()),
                vec![
                    article::slug::set(Self::slug_hash(slug.as_str()).to_string()),
                    article::deleted_at::set(Some(chrono::Utc::now().into())),
                ],
            )
            .exec()
            .await?;

        Ok(Json::from("Article deleted".to_string()))
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

        if let Some(user) = auth_user.0 {
            let favorited = Self::check_favorited(&prisma, &user, article.id).await?;

            let followed =
                ProfilesService::check_following(&prisma, &user, article.author_id).await?;

            return Ok(Json::from(ArticleBody {
                article: article.to_article(favorited, followed),
            }));
        }

        Ok(Json::from(ArticleBody {
            article: article.to_article(false, false),
        }))
    }

    pub async fn get_articles(
        auth_user: OptionalAuthUser,
        prisma: Prisma,
        Query(query): Query<ArticleListQuery>,
    ) -> Result<Json<ArticlesBody<Article>>, AppError> {
        let mut filter: Vec<prisma::article::WhereParam> = Vec::new();

        if let Some(tag) = query.tag {
            filter.push(article::tags::some(vec![article_tag::tag::equals(tag)]))
        }

        if let Some(author) = query.author {
            filter.push(article::author::is(vec![user::username::equals(author)]))
        }

        if let Some(favorited) = query.favorited {
            filter.push(article::favorited_by::some(vec![
                user_favorite_article::user::is(vec![user::username::equals(favorited)]),
            ]))
        }

        filter.push(article::deleted_at::equals(None));

        let _articles = prisma
            .article()
            .find_many(filter.clone())
            .with(article::author::fetch())
            .with(article::tags::fetch(vec![]))
            .take(query.limit.unwrap_or(20))
            .skip(query.offset.unwrap_or(0))
            .order_by(article::created_at::order(
                prisma_client_rust::Direction::Desc,
            ))
            .exec()
            .await?;

        let articles_count = prisma.article().count(filter).exec().await?;

        let mut articles: Vec<Article> = Vec::new();

        if let Some(user) = auth_user.0 {
            for article in _articles.iter() {
                let favorited = Self::check_favorited(&prisma, &user, article.id).await?;

                let followed =
                    ProfilesService::check_following(&prisma, &user, article.author_id).await?;

                articles.push(article.clone().to_article(favorited, followed));
            }
        } else {
            articles = _articles
                .iter()
                .map(|article| article.clone().to_article(false, false))
                .collect();
        }

        Ok(Json::from(ArticlesBody {
            articles,
            articles_count: articles_count as usize,
        }))
    }

    pub async fn get_articles_feed(
        auth_user: AuthUser,
        prisma: Prisma,
        Query(query): Query<ArticleListQuery>,
    ) -> Result<Json<ArticlesBody<Article>>, AppError> {
        let mut filter: Vec<prisma::article::WhereParam> = Vec::new();

        if let Some(tag) = query.tag {
            filter.push(article::tags::some(vec![article_tag::tag::equals(tag)]))
        }

        if let Some(author) = query.author {
            filter.push(article::author::is(vec![user::username::equals(author)]))
        }

        if let Some(favorited) = query.favorited {
            filter.push(article::favorited_by::some(vec![
                user_favorite_article::user::is(vec![user::username::equals(favorited)]),
            ]))
        }

        filter.push(article::author::is(vec![user::followed_by::some(vec![
            followed_by_id::equals(auth_user.user_id),
        ])]));

        filter.push(article::deleted_at::equals(None));

        let _articles = prisma
            .article()
            .find_many(filter.clone())
            .with(article::author::fetch())
            .with(article::tags::fetch(vec![]))
            .take(query.limit.unwrap_or(20))
            .skip(query.offset.unwrap_or(0))
            .order_by(article::created_at::order(Direction::Desc))
            .exec()
            .await?;

        let articles_count = prisma.article().count(filter).exec().await?;

        let mut articles: Vec<Article> = Vec::new();

        for article in _articles.iter() {
            let favorited = Self::check_favorited(&prisma, &auth_user, article.id).await?;

            let followed =
                ProfilesService::check_following(&prisma, &auth_user, article.author_id).await?;

            articles.push(article.clone().to_article(favorited, followed));
        }

        Ok(Json::from(ArticlesBody {
            articles,
            articles_count: articles_count as usize,
        }))
    }

    pub async fn favorite_article(
        auth_user: AuthUser,
        prisma: Prisma,
        Path(slug): Path<String>,
    ) -> Result<Json<ArticleBody<Article>>, AppError> {
        let article = prisma
            .article()
            .find_unique(article::slug::equals(slug.clone()))
            .with(article::author::fetch())
            .exec()
            .await?
            .ok_or(AppError::NotFound(String::from("Article not found")))?;

        let _ = prisma
            .user_favorite_article()
            .create(
                user::id::equals(auth_user.user_id),
                article::id::equals(article.id),
                vec![],
            )
            .exec()
            .await?;

        let article = prisma
            .article()
            .update(
                article::slug::equals(slug),
                vec![article::favorites_count::increment(1)],
            )
            .with(article::author::fetch())
            .exec()
            .await?;

        let following =
            ProfilesService::check_following(&prisma, &auth_user, article.author_id).await?;

        Ok(Json::from(ArticleBody {
            article: article.to_article(true, following),
        }))
    }

    pub async fn unfavorite_article(
        auth_user: AuthUser,
        prisma: Prisma,
        Path(slug): Path<String>,
    ) -> Result<Json<ArticleBody<Article>>, AppError> {
        let article = prisma
            .article()
            .find_unique(article::slug::equals(slug.clone()))
            .with(article::author::fetch())
            .exec()
            .await?
            .ok_or(AppError::NotFound(String::from("Article not found")))?;

        let _ = prisma
            .user_favorite_article()
            .delete(user_favorite_article::user_id_article_id(
                auth_user.user_id,
                article.id,
            ))
            .exec()
            .await?;

        let article = prisma
            .article()
            .update(
                article::slug::equals(slug),
                vec![article::favorites_count::decrement(1)],
            )
            .with(article::author::fetch())
            .exec()
            .await?;

        let following =
            ProfilesService::check_following(&prisma, &auth_user, article.author_id).await?;

        Ok(Json::from(ArticleBody {
            article: article.to_article(false, following),
        }))
    }

    pub async fn create_comment(
        auth_user: AuthUser,
        prisma: Prisma,
        Path(slug): Path<String>,
        Json(input): Json<CommentBody<CommentCreateInput>>,
    ) -> Result<Json<CommentBody<Comment>>, AppError> {
        let CommentBody {
            comment: CommentCreateInput { body },
        } = input;

        let article = prisma
            .article()
            .find_unique(article::slug::equals(slug.clone()))
            .with(article::author::fetch())
            .exec()
            .await?
            .ok_or(AppError::NotFound(String::from("Article not found")))?;

        let comment = prisma
            .comment()
            .create(
                body,
                user::id::equals(auth_user.user_id),
                article::id::equals(article.id),
                vec![],
            )
            .with(comment::author::fetch())
            .exec()
            .await?;

        Ok(Json::from(CommentBody {
            comment: comment.to_comment(false),
        }))
    }

    pub async fn get_comments(
        auth_user: OptionalAuthUser,
        prisma: Prisma,
        Path(slug): Path<String>,
    ) -> Result<Json<CommentsBody<Comment>>, AppError> {
        let article = prisma
            .article()
            .find_unique(article::slug::equals(slug.clone()))
            .with(article::author::fetch())
            .exec()
            .await?
            .ok_or(AppError::NotFound(String::from("Article not found")))?;

        let comments = prisma
            .comment()
            .find_many(vec![
                comment::article_id::equals(article.id),
                comment::deleted_at::equals(None),
            ])
            .with(comment::author::fetch())
            .exec()
            .await?;

        let mut comments: Vec<Comment> = comments
            .iter()
            .map(|comment| comment.clone().to_comment(false))
            .collect();

        if let Some(user) = auth_user.0 {
            for comment in comments.iter_mut() {
                let followed =
                    ProfilesService::check_following(&prisma, &user, article.author_id).await?;

                comment.author.following = followed;
            }
        }

        Ok(Json::from(CommentsBody { comments }))
    }

    pub async fn delete_comment(
        auth_user: AuthUser,
        prisma: Prisma,
        Path((_slug, comment_id)): Path<(String, i32)>,
    ) -> Result<Json<String>, AppError> {
        let comment = prisma
            .comment()
            .find_unique(comment::id::equals(comment_id))
            .with(comment::author::fetch())
            .exec()
            .await?
            .ok_or(AppError::NotFound(String::from("Comment not found")))?;

        if comment.author_id != auth_user.user_id {
            return Err(AppError::BadRequest(String::from(
                "You are not the author of this comment",
            )));
        }

        let _ = prisma
            .comment()
            .update(
                comment::id::equals(comment_id),
                vec![comment::deleted_at::set(Some(chrono::Utc::now().into()))],
            )
            .exec()
            .await?;

        Ok(Json::from("Comment deleted".to_string()))
    }
}
