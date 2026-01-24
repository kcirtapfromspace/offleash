use chrono::{DateTime, Utc};
use shared::types::{BookingId, OrganizationId, RecurringBookingSeriesId, UserId};
use sqlx::{PgPool, Postgres, Transaction};

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
            INSERT INTO bookings (id, organization_id, customer_id, walker_id, service_id, location_id, scheduled_start, scheduled_end, price_cents, notes, recurring_series_id, occurrence_number)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, recurring_series_id, occurrence_number, created_at, updated_at
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
        .bind(input.recurring_series_id.map(|id| *id.as_uuid()))
        .bind(input.occurrence_number)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(booking)
    }

    /// Create a booking within an existing transaction (for batch operations)
    /// Skips conflict check since caller is responsible for pre-checking
    pub async fn create_in_tx(
        tx: &mut Transaction<'_, Postgres>,
        input: CreateBooking,
    ) -> Result<Booking, sqlx::Error> {
        let id = BookingId::new();

        sqlx::query_as::<_, Booking>(
            r#"
            INSERT INTO bookings (id, organization_id, customer_id, walker_id, service_id, location_id, scheduled_start, scheduled_end, price_cents, notes, recurring_series_id, occurrence_number)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, recurring_series_id, occurrence_number, created_at, updated_at
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
        .bind(input.recurring_series_id.map(|id| *id.as_uuid()))
        .bind(input.occurrence_number)
        .fetch_one(&mut **tx)
        .await
    }

    pub async fn find_by_id(
        pool: &PgPool,
        org_id: OrganizationId,
        id: BookingId,
    ) -> Result<Option<Booking>, sqlx::Error> {
        sqlx::query_as::<_, Booking>(
            r#"
            SELECT id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, recurring_series_id, occurrence_number, created_at, updated_at
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
            SELECT id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, recurring_series_id, occurrence_number, created_at, updated_at
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
            SELECT id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, recurring_series_id, occurrence_number, created_at, updated_at
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

    /// Find all bookings assigned to a walker
    pub async fn find_by_walker(
        pool: &PgPool,
        org_id: OrganizationId,
        walker_id: UserId,
    ) -> Result<Vec<Booking>, sqlx::Error> {
        sqlx::query_as::<_, Booking>(
            r#"
            SELECT id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, recurring_series_id, occurrence_number, created_at, updated_at
            FROM bookings
            WHERE walker_id = $1 AND organization_id = $2
            ORDER BY scheduled_start DESC
            "#,
        )
        .bind(walker_id.as_uuid())
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
            RETURNING id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, recurring_series_id, occurrence_number, created_at, updated_at
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

    /// Reschedule a booking to a new time
    pub async fn reschedule(
        pool: &PgPool,
        org_id: OrganizationId,
        id: BookingId,
        new_start: chrono::DateTime<chrono::Utc>,
        new_end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Option<Booking>, sqlx::Error> {
        sqlx::query_as::<_, Booking>(
            r#"
            UPDATE bookings
            SET scheduled_start = $3,
                scheduled_end = $4,
                updated_at = NOW()
            WHERE id = $1
              AND organization_id = $2
              AND status IN ('pending', 'confirmed')
            RETURNING *
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(new_start)
        .bind(new_end)
        .fetch_optional(pool)
        .await
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
                    SELECT id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, recurring_series_id, occurrence_number, created_at, updated_at
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
                    SELECT id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, recurring_series_id, occurrence_number, created_at, updated_at
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

    /// Find all bookings in a recurring series
    pub async fn find_by_series(
        pool: &PgPool,
        org_id: OrganizationId,
        series_id: RecurringBookingSeriesId,
    ) -> Result<Vec<Booking>, sqlx::Error> {
        sqlx::query_as::<_, Booking>(
            r#"
            SELECT id, organization_id, customer_id, walker_id, service_id, location_id, status, scheduled_start, scheduled_end, actual_start, actual_end, price_cents, notes, recurring_series_id, occurrence_number, created_at, updated_at
            FROM bookings
            WHERE recurring_series_id = $1 AND organization_id = $2
            ORDER BY scheduled_start ASC
            "#,
        )
        .bind(series_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    /// Cancel all future bookings in a recurring series
    pub async fn cancel_future_by_series(
        pool: &PgPool,
        org_id: OrganizationId,
        series_id: RecurringBookingSeriesId,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE bookings
            SET status = 'cancelled', updated_at = NOW()
            WHERE recurring_series_id = $1
              AND organization_id = $2
              AND status IN ('pending', 'confirmed')
              AND scheduled_start > NOW()
            "#,
        )
        .bind(series_id.as_uuid())
        .bind(org_id.as_uuid())
        .execute(pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }

    /// Cancel all bookings in a recurring series (entire series)
    pub async fn cancel_all_by_series(
        pool: &PgPool,
        org_id: OrganizationId,
        series_id: RecurringBookingSeriesId,
    ) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE bookings
            SET status = 'cancelled', updated_at = NOW()
            WHERE recurring_series_id = $1
              AND organization_id = $2
              AND status IN ('pending', 'confirmed')
            "#,
        )
        .bind(series_id.as_uuid())
        .bind(org_id.as_uuid())
        .execute(pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}
