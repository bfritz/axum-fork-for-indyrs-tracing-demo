use sqlx::postgres::PgPool;

use crate::Pagination;
use crate::Todo;

pub async fn find_all_todos(pool: PgPool, pagination: Pagination) -> anyhow::Result<Vec<Todo>> {
    let limit: i64 = pagination.limit.unwrap_or(i64::MAX);
    let offset: i64 = pagination.offset.unwrap_or(0);
    let todos = sqlx::query_as!(Todo,
        r#"
SELECT id, text, completed
FROM todos
LIMIT $1
OFFSET $2
        "#,
        limit,
        offset,
        )
        .fetch_all(&pool)
        .await?;

    Ok(todos)
}

pub async fn find_one_todo(pool: &PgPool, id: uuid::Uuid) -> anyhow::Result<Option<Todo>> {
    let todo = sqlx::query_as!(Todo,
        r#"
SELECT id, text, completed
FROM todos
WHERE id = $1
       "#,
       id,
       )
        .fetch_optional(pool)
        .await?;
    Ok(todo)
}

pub async fn insert_todo(pool: PgPool, todo: Todo) -> anyhow::Result<Todo> {
    let todo = sqlx::query_as!(Todo,
        r#"
INSERT INTO todos (id, text, completed)
VALUES ($1, $2, $3)
RETURNING id, text, completed
       "#,
       todo.id,
       todo.text,
       todo.completed,
       )
        .fetch_one(&pool)
        .await?;
    Ok(todo)
}

pub async fn update_todo(pool: PgPool, todo: Todo) -> anyhow::Result<Todo> {
    let todo = sqlx::query_as!(Todo,
        r#"
UPDATE todos SET
  text = $2,
  completed = $3
WHERE id = $1
RETURNING id, text, completed
       "#,
       todo.id,
       todo.text,
       todo.completed,
       )
        .fetch_one(&pool)
        .await?;
    Ok(todo)
}

pub async fn delete_todo(pool: PgPool, id: uuid::Uuid) -> anyhow::Result<Option<uuid::Uuid>> {
    let delete_count = sqlx::query!("DELETE FROM todos WHERE id = $1", id)
        .execute(&pool)
        .await?
        .rows_affected();

    if delete_count > 0 {
        Ok(Some(id))
    } else {
        Ok(None)
    }
}
