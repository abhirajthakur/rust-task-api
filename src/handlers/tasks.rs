use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde_json::{Value, json};
use sqlx::Row;

use crate::{
    models::{CreateTaskRequest, Task, TaskQuery, UpdateTaskRequest},
    state::AppState,
};

pub async fn list_tasks(
    State(state): State<AppState>,
    Query(params): Query<TaskQuery>,
) -> Result<Json<Vec<Task>>, StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, description, completed
        FROM tasks
        WHERE ($1::BOOLEAN IS NULL OR completed = $1)
        ORDER BY id
        "#,
    )
    .bind(params.completed)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tasks: Vec<Task> = rows
        .iter()
        .map(|row| Task {
            id: row.get::<i64, _>("id") as u64,
            title: row.get("title"),
            description: row.get("description"),
            completed: row.get("completed"),
        })
        .collect();

    Ok(Json(tasks))
}

pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), (StatusCode, Json<Value>)> {
    if payload.title.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Task title cannot be empty"
            })),
        ));
    }

    let row = sqlx::query(
        r#"
        INSERT INTO tasks (title, description)
        VALUES ($1, $2)
        RETURNING id, title, description, completed
        "#,
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to create task"
            })),
        )
    })?;

    let task = Task {
        id: row.get::<i64, _>("id") as u64,
        title: row.get("title"),
        description: row.get("description"),
        completed: row.get("completed"),
    };

    Ok((StatusCode::CREATED, Json(task)))
}

pub async fn get_task(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<Task>, StatusCode> {
    let row = sqlx::query(
        r#"
        SELECT id, title, description, completed
        FROM tasks
        WHERE (id = $1)
        "#,
    )
    .bind(id as i64)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some(row) = row else {
        return Err(StatusCode::NOT_FOUND);
    };

    let task = Task {
        id: row.get::<i64, _>("id") as u64,
        title: row.get("title"),
        description: row.get("description"),
        completed: row.get("completed"),
    };

    Ok(Json(task))
}

pub async fn update_task(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, (StatusCode, Json<Value>)> {
    if payload
        .title
        .as_ref()
        .is_some_and(|title| title.trim().is_empty())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Task title cannot be empty"
            })),
        ));
    }

    let row = sqlx::query(
        r#"
            UPDATE tasks
            SET
                title = COALESCE($1, title),
                description = COALESCE($2, description),
                completed = COALESCE($3, completed)
            WHERE id = $4
            RETURNING id, title, description, completed
            "#,
    )
    .bind(payload.title)
    .bind(payload.description)
    .bind(payload.completed)
    .bind(id as i64)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to update task"
            })),
        )
    })?;

    let Some(row) = row else {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Task not found"
            })),
        ));
    };

    let task = Task {
        id: row.get::<i64, _>("id") as u64,
        title: row.get("title"),
        description: row.get("description"),
        completed: row.get("completed"),
    };

    Ok(Json(task))
}

pub async fn delete_task(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = $1")
        .bind(id as i64)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
