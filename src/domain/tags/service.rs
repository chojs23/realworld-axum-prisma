use axum::{Extension, Json};
use std::sync::Arc;

use crate::{app_error::AppError, prisma::PrismaClient};

use super::TagsBody;

type Prisma = Extension<Arc<PrismaClient>>;

pub struct TagsService;

impl TagsService {
    pub async fn get_tags(prisma: Prisma) -> Result<Json<TagsBody>, AppError> {
        let tags = prisma.article_tag().find_many(vec![]).exec().await.unwrap();

        let tags = tags.into_iter().map(|tag| tag.tag).collect();

        Ok(Json::from(TagsBody { tags }))
    }
}
