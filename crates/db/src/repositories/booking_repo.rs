use chrono::{DateTime, Utc};
use shared::types::{BookingId, OrganizationId, UserId};
use sqlx::PgPool;

use crate::models::{Booking, BookingStatus, CreateBooking};

pub struct BookingRepository;

impl BookingRepository {
    /// Create a booking with transaction locking to prevent double-booking
    pub async fn create(pool: &PgPool, input: CreateBooking) -> Result<Booking, sqlx::Error> {
        let id = BookingId::new();

        // Start a transaction
        let mut tx = pool.begin().await?;

        // Lock by walker to prevent concurrent double-booking
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1::text))")
            .bind(input.walker_id.as_uuid().to_string())
            .execute(&mut *tx)
            .await?;

        // Check for conflicts with existing bookings
        let conflicts: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM bookings
            WHERE walker_id = $1
              AND organization_id = $2
              AND status NOT IN ('cancelled', 'completed')
              AND scheduled_start < $4
              AND scheduled_end > $3
            "#,
        )
        .bind(input.walker_id.as_uuid())
        .bind(input.organization_id.as_uuid())
        .bind(input.scheduled_start)
        .bind(input.scheduled_end)
        .fetch_one(&mut *tx)
        .await?;

        if conflicts.0 > 0 {
            tx.rollback().await?;
            return Err(sqlx::Error::Protocol("Time slot conflict".to_string()));
        }

        // Insert the booking
        let booking = sqlx::query_as::<_, Booking>(
            r#"
            INSERT INTO bookings (id, organization_id, customer_id, walker_id, service_id, location_id, scheduled_start, scheduled_end, price_cents, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(input.organization_id.as_uuid())
        .bind(input.customer_id.as_uuid())
        .bind(input.walker_id.as_uuid())
        .bind(input.service_id.as_uuid())
        .bind(input.location_id.as_uuid())
        .bind(input.scheduled_start)
        .bind(input.scheduled_end)
        .bind(input.price_cents)
        .bind(&input.notes)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(booking)
    }

    pub async fn find_by_id(
        pool: &PgPool,
        org_id: OrganizationId,
        id: BookingId,
    ) -> Result<Option<Booking>, sqlx::Error> {
        sqlx::query_as::<_, Booking>(
            r#"
            SELECT id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, created_at, updated_at
            FROM bookings
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_walker_in_range(
        pool: &PgPool,
        org_id: OrganizationId,
        walker_id: UserId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Booking>, sqlx::Error> {
        sqlx::query_as::<_, Booking>(
            r#"
            SELECT id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, created_at, updated_at
            FROM bookings
            WHERE walker_id = $1
              AND organization_id = $2
              AND scheduled_start < $4
              AND scheduled_end > $3
              AND status NOT IN ('cancelled')
            ORDER BY scheduled_start
            "#,
        )
        .bind(walker_id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(start)
        .bind(end)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_customer(
        pool: &PgPool,
        org_id: OrganizationId,
        customer_id: UserId,
    ) -> Result<Vec<Booking>, sqlx::Error> {
        sqlx::query_as::<_, Booking>(
            r#"
            SELECT id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, created_at, updated_at
            FROM bookings
            WHERE customer_id = $1 AND organization_id = $2
            ORDER BY scheduled_start DESC
            "#,
        )
        .bind(customer_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    pub async fn update_status(
        pool: &PgPool,
        org_id: OrganizationId,
        id: BookingId,
        status: BookingStatus,
    ) -> Result<Option<Booking>, sqlx::Error> {
        sqlx::query_as::<_, Booking>(
            r#"
            UPDATE bookings
            SET status = $3, updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(status)
        .fetch_optional(pool)
        .await
    }

    pub async fn confirm(
        pool: &PgPool,
        org_id: OrganizationId,
        id: BookingId,
    ) -> Result<Option<Booking>, sqlx::Error> {
        Self::update_status(pool, org_id, id, BookingStatus::Confirmed).await
    }

    pub async fn cancel(
        pool: &PgPool,
        org_id: OrganizationId,
        id: BookingId,
    ) -> Result<Option<Booking>, sqlx::Error> {
        Self::update_status(pool, org_id, id, BookingStatus::Cancelled).await
    }

    /// Count bookings for today
    pub async fn count_today(pool: &PgPool, org_id: OrganizationId) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM bookings
            WHERE organization_id = $1
              AND DATE(scheduled_start) = CURRENT_DATE
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }

    /// Count pending bookings
    pub async fn count_pending(pool: &PgPool, org_id: OrganizationId) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM bookings
            WHERE organization_id = $1
              AND status = 'pending'
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }

    /// List all bookings for an organization, optionally filtered by status
    pub async fn list_all(
        pool: &PgPool,
        org_id: OrganizationId,
        status_filter: Option<BookingStatus>,
    ) -> Result<Vec<Booking>, sqlx::Error> {
        match status_filter {
            Some(status) => {
                sqlx::query_as::<_, Booking>(
                    r#"
                    SELECT id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, created_at, updated_at
                    FROM bookings
                    WHERE organization_id = $1 AND status = $2
                    ORDER BY scheduled_start DESC
                    "#,
                )
                .bind(org_id.as_uuid())
                .bind(status)
                .fetch_all(pool)
                .await
            }
            None => {
                sqlx::query_as::<_, Booking>(
                    r#"
                    SELECT id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, created_at, updated_at
                    FROM bookings
                    WHERE organization_id = $1
                    ORDER BY scheduled_start DESC
                    "#,
                )
                .bind(org_id.as_uuid())
                .fetch_all(pool)
                .await
            }
        }
    }

    /// Complete a booking
    pub async fn complete(
        pool: &PgPool,
        org_id: OrganizationId,
        id: BookingId,
    ) -> Result<Option<Booking>, sqlx::Error> {
        Self::update_status(pool, org_id, id, BookingStatus::Completed).await
    }
}
