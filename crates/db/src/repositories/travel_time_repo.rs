use chrono::{Duration, Utc};
use shared::types::{LocationId, UserId};
use sqlx::PgPool;

use crate::models::{TravelTimeCache, WalkerLocation, WalkerLocationUpdate};

pub struct TravelTimeCacheRepository;

impl TravelTimeCacheRepository {
    /// Get cached travel time between two locations
    pub async fn get(
        pool: &PgPool,
        origin_id: LocationId,
        destination_id: LocationId,
    ) -> Result<Option<TravelTimeCache>, sqlx::Error> {
        let cache = sqlx::query_as::<_, TravelTimeCache>(
            r#"
            SELECT id, origin_location_id, destination_location_id,
                   travel_seconds, distance_meters, calculated_at
            FROM travel_time_cache
            WHERE origin_location_id = $1 AND destination_location_id = $2
            "#,
        )
        .bind(origin_id)
        .bind(destination_id)
        .fetch_optional(pool)
        .await?;

        Ok(cache)
    }

    /// Get cached travel time if not stale (within max_age_minutes)
    pub async fn get_if_fresh(
        pool: &PgPool,
        origin_id: LocationId,
        destination_id: LocationId,
        max_age_minutes: i64,
    ) -> Result<Option<TravelTimeCache>, sqlx::Error> {
        let cutoff = Utc::now() - Duration::minutes(max_age_minutes);

        let cache = sqlx::query_as::<_, TravelTimeCache>(
            r#"
            SELECT id, origin_location_id, destination_location_id,
                   travel_seconds, distance_meters, calculated_at
            FROM travel_time_cache
            WHERE origin_location_id = $1
              AND destination_location_id = $2
              AND calculated_at > $3
            "#,
        )
        .bind(origin_id)
        .bind(destination_id)
        .bind(cutoff)
        .fetch_optional(pool)
        .await?;

        Ok(cache)
    }

    /// Upsert travel time cache entry
    pub async fn upsert(
        pool: &PgPool,
        origin_id: LocationId,
        destination_id: LocationId,
        travel_seconds: i32,
        distance_meters: i32,
    ) -> Result<TravelTimeCache, sqlx::Error> {
        let cache = sqlx::query_as::<_, TravelTimeCache>(
            r#"
            INSERT INTO travel_time_cache
                (origin_location_id, destination_location_id, travel_seconds, distance_meters, calculated_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (origin_location_id, destination_location_id)
            DO UPDATE SET
                travel_seconds = EXCLUDED.travel_seconds,
                distance_meters = EXCLUDED.distance_meters,
                calculated_at = NOW()
            RETURNING id, origin_location_id, destination_location_id,
                      travel_seconds, distance_meters, calculated_at
            "#,
        )
        .bind(origin_id)
        .bind(destination_id)
        .bind(travel_seconds)
        .bind(distance_meters)
        .fetch_one(pool)
        .await?;

        Ok(cache)
    }

    /// Delete stale cache entries
    pub async fn delete_stale(pool: &PgPool, max_age_minutes: i64) -> Result<u64, sqlx::Error> {
        let cutoff = Utc::now() - Duration::minutes(max_age_minutes);

        let result = sqlx::query(
            r#"
            DELETE FROM travel_time_cache
            WHERE calculated_at < $1
            "#,
        )
        .bind(cutoff)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

pub struct WalkerLocationRepository;

impl WalkerLocationRepository {
    /// Get walker's current location
    pub async fn get(
        pool: &PgPool,
        walker_id: UserId,
    ) -> Result<Option<WalkerLocation>, sqlx::Error> {
        let location = sqlx::query_as::<_, WalkerLocation>(
            r#"
            SELECT walker_id, latitude, longitude, accuracy_meters,
                   heading, speed_mps, is_on_duty, updated_at, created_at
            FROM walker_locations
            WHERE walker_id = $1
            "#,
        )
        .bind(walker_id)
        .fetch_optional(pool)
        .await?;

        Ok(location)
    }

    /// Get walker's location if fresh (within max_age_minutes)
    pub async fn get_if_fresh(
        pool: &PgPool,
        walker_id: UserId,
        max_age_minutes: i64,
    ) -> Result<Option<WalkerLocation>, sqlx::Error> {
        let cutoff = Utc::now() - Duration::minutes(max_age_minutes);

        let location = sqlx::query_as::<_, WalkerLocation>(
            r#"
            SELECT walker_id, latitude, longitude, accuracy_meters,
                   heading, speed_mps, is_on_duty, updated_at, created_at
            FROM walker_locations
            WHERE walker_id = $1 AND updated_at > $2
            "#,
        )
        .bind(walker_id)
        .bind(cutoff)
        .fetch_optional(pool)
        .await?;

        Ok(location)
    }

    /// Get all on-duty walkers with fresh locations
    pub async fn get_on_duty(
        pool: &PgPool,
        max_age_minutes: i64,
    ) -> Result<Vec<WalkerLocation>, sqlx::Error> {
        let cutoff = Utc::now() - Duration::minutes(max_age_minutes);

        let locations = sqlx::query_as::<_, WalkerLocation>(
            r#"
            SELECT walker_id, latitude, longitude, accuracy_meters,
                   heading, speed_mps, is_on_duty, updated_at, created_at
            FROM walker_locations
            WHERE is_on_duty = true AND updated_at > $1
            "#,
        )
        .bind(cutoff)
        .fetch_all(pool)
        .await?;

        Ok(locations)
    }

    /// Upsert walker location
    pub async fn upsert(
        pool: &PgPool,
        walker_id: UserId,
        update: &WalkerLocationUpdate,
    ) -> Result<WalkerLocation, sqlx::Error> {
        let location = sqlx::query_as::<_, WalkerLocation>(
            r#"
            INSERT INTO walker_locations
                (walker_id, latitude, longitude, accuracy_meters, heading, speed_mps, is_on_duty, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            ON CONFLICT (walker_id)
            DO UPDATE SET
                latitude = EXCLUDED.latitude,
                longitude = EXCLUDED.longitude,
                accuracy_meters = EXCLUDED.accuracy_meters,
                heading = EXCLUDED.heading,
                speed_mps = EXCLUDED.speed_mps,
                is_on_duty = EXCLUDED.is_on_duty,
                updated_at = NOW()
            RETURNING walker_id, latitude, longitude, accuracy_meters,
                      heading, speed_mps, is_on_duty, updated_at, created_at
            "#,
        )
        .bind(walker_id)
        .bind(update.latitude)
        .bind(update.longitude)
        .bind(update.accuracy_meters)
        .bind(update.heading)
        .bind(update.speed_mps)
        .bind(update.is_on_duty)
        .fetch_one(pool)
        .await?;

        Ok(location)
    }

    /// Update on-duty status only
    pub async fn set_on_duty(
        pool: &PgPool,
        walker_id: UserId,
        is_on_duty: bool,
    ) -> Result<Option<WalkerLocation>, sqlx::Error> {
        let location = sqlx::query_as::<_, WalkerLocation>(
            r#"
            UPDATE walker_locations
            SET is_on_duty = $2, updated_at = NOW()
            WHERE walker_id = $1
            RETURNING walker_id, latitude, longitude, accuracy_meters,
                      heading, speed_mps, is_on_duty, updated_at, created_at
            "#,
        )
        .bind(walker_id)
        .bind(is_on_duty)
        .fetch_optional(pool)
        .await?;

        Ok(location)
    }

    /// Delete walker location
    pub async fn delete(pool: &PgPool, walker_id: UserId) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM walker_locations
            WHERE walker_id = $1
            "#,
        )
        .bind(walker_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Create location from coordinates (for ad-hoc locations not in locations table)
    #[allow(dead_code)]
    pub fn from_coordinates(
        _walker_id: UserId,
        latitude: f64,
        longitude: f64,
    ) -> WalkerLocationUpdate {
        WalkerLocationUpdate {
            latitude,
            longitude,
            accuracy_meters: None,
            heading: None,
            speed_mps: None,
            is_on_duty: true,
        }
    }
}
