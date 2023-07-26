use std::marker::PhantomData;

use crate::{Pool, QueryResult, Result, Row};

pub struct Executor<'a, T>(&'a Pool, PhantomData<T>)
where
    T: Send + Unpin + for<'r> sqlx::FromRow<'r, Row>;

impl<'a, T> Executor<'a, T>
where
    T: Send + Unpin + for<'r> sqlx::FromRow<'r, Row>,
{
    pub fn from(pool: &'a Pool) -> Self {
        Self(pool, PhantomData)
    }

    pub async fn fetch_one(&self, query: &str, params: &Vec<String>) -> Result<T> {
        let query = sqlx::query_as(query);
        let query = params.iter().fold(query, |query, param| query.bind(param));
        query.fetch_one(self.0).await
    }

    pub async fn fetch_all(&self, query: &str, params: &Vec<String>) -> Result<Vec<T>> {
        let query = sqlx::query_as(query);
        let query = params.iter().fold(query, |query, param| query.bind(param));
        query.fetch_all(self.0).await
    }

    pub async fn execute(&self, query: &str, params: &Vec<String>) -> Result<QueryResult> {
        let query = sqlx::query(query);
        let query = params.iter().fold(query, |query, param| query.bind(param));
        query.execute(self.0).await
    }
}
