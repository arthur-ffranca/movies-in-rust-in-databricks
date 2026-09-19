use csv::StringRecord;
use sqlx::{Row, postgres::PgPoolOptions, query};
use std::error::Error;
use std::path::PathBuf;

const SAMPLES_DIR: &str = "../mongodbatlas-samples";

fn sample_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(SAMPLES_DIR)
        .join(file_name)
}

fn get<'a>(headers: &StringRecord, row: &'a StringRecord, name: &str) -> &'a str {
    headers
        .iter()
        .position(|header| header == name)
        .and_then(|index| row.get(index))
        .unwrap_or("")
}

fn optional(value: &str) -> Option<&str> {
    (!value.trim().is_empty()).then_some(value.trim())
}

fn number_i64(value: &str) -> Option<i64> {
    value.trim().parse().ok()
}

fn number_f64(value: &str) -> Option<f64> {
    value.trim().parse().ok()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let database_url = std::env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let mut movies_csv = csv::Reader::from_path(sample_path("sample_mflix.movies.csv"))?;
    let movie_headers = movies_csv.headers()?.clone();
    let mut movie_count = 0_u64;

    for result in movies_csv.records() {
        let row = result?;

        let movie_id = get(&movie_headers, &row, "_id");
        let title = get(&movie_headers, &row, "title");

        if movie_id.is_empty() || title.is_empty() {
            continue;
        }

        let genres: Vec<String> = movie_headers
            .iter()
            .enumerate()
            .filter(|(_, header)| header.starts_with("genres["))
            .filter_map(|(index, _)| row.get(index))
            .filter_map(optional)
            .map(str::to_string)
            .collect();

        query(
            r#"
            INSERT INTO movie_lab.public.movies (
                movie_id, title, plot, fullplot, genres, runtime, rated,
                released, year, imdb_rating, imdb_votes, poster
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7,
                NULLIF($8, '')::TIMESTAMPTZ, $9, $10, $11, $12
            )
            ON CONFLICT (movie_id) DO UPDATE SET
                title = EXCLUDED.title,
                plot = EXCLUDED.plot,
                fullplot = EXCLUDED.fullplot,
                genres = EXCLUDED.genres,
                runtime = EXCLUDED.runtime,
                rated = EXCLUDED.rated,
                released = EXCLUDED.released,
                year = EXCLUDED.year,
                imdb_rating = EXCLUDED.imdb_rating,
                imdb_votes = EXCLUDED.imdb_votes,
                poster = EXCLUDED.poster
            "#,
        )
        .bind(movie_id)
        .bind(title)
        .bind(optional(get(&movie_headers, &row, "plot")))
        .bind(optional(get(&movie_headers, &row, "fullplot")))
        .bind(genres)
        .bind(number_i64(get(&movie_headers, &row, "runtime")))
        .bind(optional(get(&movie_headers, &row, "rated")))
        .bind(get(&movie_headers, &row, "released"))
        .bind(number_i64(get(&movie_headers, &row, "year")))
        .bind(number_f64(get(&movie_headers, &row, "imdb.rating")))
        .bind(number_i64(get(&movie_headers, &row, "imdb.votes")))
        .bind(optional(get(&movie_headers, &row, "poster")))
        .execute(&pool)
        .await?;

        movie_count += 1;
    }

    let mut users_csv = csv::Reader::from_path(sample_path("sample_mflix.users.csv"))?;
    let user_headers = users_csv.headers()?.clone();
    let mut user_count = 0_u64;

    for result in users_csv.records() {
        let row = result?;

        let user_id = get(&user_headers, &row, "_id");
        let name = get(&user_headers, &row, "name");
        let email = get(&user_headers, &row, "email");

        if user_id.is_empty() || name.is_empty() || email.is_empty() {
            continue;
        }

        let _result1 = query(
            r#"
            INSERT INTO movie_lab.public.users (user_id, name, email)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id) DO UPDATE SET
                name = EXCLUDED.name,
                email = EXCLUDED.email
            "#,
        )
        .bind(user_id)
        .bind(name)
        .bind(email)
        .execute(&pool)
        .await?;

        user_count += 1;
    }

    let movie_total: i64 = query("SELECT count(*) AS total FROM movie_lab.public.movies")
        .fetch_one(&pool)
        .await?
        .get("total");

    let user_total: i64 = query("SELECT count(*) AS total FROM movie_lab.public.users")
        .fetch_one(&pool)
        .await?
        .get("total");

    println!("CSV processado | filmes: {movie_count} | usuários: {user_count}");
    println!("Barata confirmou | filmes: {movie_total} | usuários: {user_total}");

    Ok(())
}
