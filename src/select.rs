use std::marker::PhantomData;

use crate::{Operation, Protocol, Result, Row};

pub struct SelectQuery<O, M> {
    base_query: &'static str,
    wheres: Vec<WhereClause>,
    params: Vec<String>,
    limit_: isize,
    out: PhantomData<O>,
    mock: PhantomData<M>,
}

// TODO: dont expose this type
pub enum WhereClause {
    Single(String),
    And(String),
    Or(String),
}

impl WhereClause {
    fn to_string(&self) -> String {
        match self {
            WhereClause::Single(clause) => clause.clone(),
            WhereClause::And(clause) => format!("AND {}", clause),
            WhereClause::Or(clause) => format!("OR {}", clause),
        }
    }
}

pub type WhereFunction<T> = fn(model: T) -> Operation;

impl<O, M> SelectQuery<O, M> {
    pub fn new(base_query: &'static str) -> Self {
        SelectQuery {
            base_query,
            params: vec![],
            wheres: vec![],
            limit_: 0,
            out: PhantomData,
            mock: PhantomData,
        }
    }

    pub fn with_params(
        base_query: &'static str,
        params: Vec<String>,
        wheres: Vec<WhereClause>,
    ) -> Self {
        SelectQuery {
            base_query,
            params,
            wheres,
            limit_: 0,
            out: PhantomData,
            mock: PhantomData,
        }
    }

    pub fn build_wheres(&self) -> String {
        if self.wheres.is_empty() {
            return String::default();
        }
        let clauses = self
            .wheres
            .iter()
            .map(|clause| clause.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        format!(" WHERE {}", clauses)
    }

    pub fn get_query(&self) -> String {
        let limit = match self.limit_ {
            0 => String::default(),
            i => format!("LIMIT {}", i),
        };
        format!("{}{} {}", self.base_query, self.build_wheres(), limit)
    }
}

// query building
impl<O, M> SelectQuery<O, M>
where
    M: Default,
{
    pub fn or(&mut self, lambda: WhereFunction<M>) -> &mut Self {
        let op = lambda(M::default());
        let (clause, mut params) = op.format();

        self.wheres.push(WhereClause::Or(clause));
        self.params.append(&mut params);

        self
    }

    pub fn and(&mut self, lambda: WhereFunction<M>) -> &mut Self {
        let op = lambda(M::default());
        let (clause, mut params) = op.format();

        self.wheres.push(WhereClause::And(clause));
        self.params.append(&mut params);

        self
    }

    pub fn limit(&mut self, limit: isize) -> &mut Self {
        self.limit_ = limit;
        self
    }
}

// executor
use crate::executor::Executor;
impl<'a, O, M> SelectQuery<O, M>
where
    O: 'a + Send + Unpin + for<'r> sqlx::FromRow<'r, Row>,
{
    pub fn get_executor<E: sqlx::Executor<'a, Database = Protocol>>(
        &self,
        executor: E,
    ) -> Executor<'a, O, E> {
        Executor::from(executor)
    }

    pub async fn fetch_one(
        &mut self,
        executor: impl sqlx::Executor<'a, Database = Protocol>,
    ) -> Result<O> {
        self.limit_ = 1;
        self.get_executor(executor)
            .fetch_one(self.get_query().as_str(), &self.params)
            .await
    }

    pub async fn fetch_all(
        &mut self,
        executor: impl sqlx::Executor<'a, Database = Protocol>,
    ) -> Result<Vec<O>> {
        self.get_executor(executor)
            .fetch_all(self.get_query().as_str(), &self.params)
            .await
    }
}
