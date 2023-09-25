use std::marker::PhantomData;

use crate::{Protocol, QueryResult, Result, Row};

pub struct Executor<
    'a,
    T: Send + Unpin + for<'r> sqlx::FromRow<'r, Row>,
    E: sqlx::Executor<'a, Database = Protocol>,
>(E, PhantomData<&'a T>);

impl<'a, T, E> Executor<'a, T, E>
where
    T: Send + Unpin + for<'r> sqlx::FromRow<'r, Row>,
    E: sqlx::Executor<'a, Database = Protocol>,
{
    pub fn from(executor: E) -> Self {
        Self(executor, PhantomData)
    }

    pub async fn fetch_one(self, query: &str, params: &Vec<String>) -> Result<T> {
        let query = sqlx::query_as(query);
        let query = params.iter().fold(query, |query, param| query.bind(param));
        query.fetch_one(self.0).await
    }

    pub async fn fetch_all(self, query: &str, params: &Vec<String>) -> Result<Vec<T>> {
        let query = sqlx::query_as(query);
        let query = params.iter().fold(query, |query, param| query.bind(param));
        query.fetch_all(self.0).await
    }

    pub async fn execute(self, query: &str, params: &Vec<String>) -> Result<QueryResult> {
        let query = sqlx::query(query);
        let query = params.iter().fold(query, |query, param| query.bind(param));
        query.execute(self.0).await
    }
}
