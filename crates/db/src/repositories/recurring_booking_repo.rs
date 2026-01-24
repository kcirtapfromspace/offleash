use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use chrono_tz::Tz;
use shared::types::{OrganizationId, RecurringBookingSeriesId, UserId};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::{
    CreateRecurringBookingSeries, OccurrenceConflict, RecurrenceFrequency, RecurringBookingSeries,
};

pub struct RecurringBookingRepository;

impl RecurringBookingRepository {
    /// Create a new recurring booking series (non-transactional for backward compatibility)
    pub async fn create(
        pool: &PgPool,
        input: CreateRecurringBookingSeries,
    ) -> Result<RecurringBookingSeries, sqlx::Error> {
        let id = RecurringBookingSeriesId::new();
        let idempotency_expires_at = input
            .idempotency_key
            .map(|_| Utc::now() + Duration::hours(24));

        sqlx::query_as::<_, RecurringBookingSeries>(
            r#"
            INSERT INTO recurring_booking_series (
                id, organization_id, customer_id, walker_id, service_id, location_id,
                frequency, day_of_week, time_of_day, timezone, end_date, total_occurrences,
                price_cents_per_booking, default_notes, idempotency_key, idempotency_expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING id, organization_id, customer_id, walker_id, service_id, location_id,
                      frequency, day_of_week, time_of_day, timezone, end_date, total_occurrences,
                      is_active, price_cents_per_booking, default_notes,
                      idempotency_key, idempotency_expires_at, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(input.organization_id.as_uuid())
        .bind(input.customer_id.as_uuid())
        .bind(input.walker_id.as_uuid())
        .bind(input.service_id.as_uuid())
        .bind(input.location_id.as_uuid())
        .bind(&input.frequency)
        .bind(input.day_of_week)
        .bind(input.time_of_day)
        .bind(&input.timezone)
        .bind(input.end_date)
        .bind(input.total_occurrences)
        .bind(input.price_cents_per_booking)
        .bind(&input.default_notes)
        .bind(input.idempotency_key)
        .bind(idempotency_expires_at)
        .fetch_one(pool)
        .await
    }

    /// Create a new recurring booking series within a transaction
    pub async fn create_in_tx(
        tx: &mut Transaction<'_, Postgres>,
        input: CreateRecurringBookingSeries,
    ) -> Result<RecurringBookingSeries, sqlx::Error> {
        let id = RecurringBookingSeriesId::new();
        let idempotency_expires_at = input
            .idempotency_key
            .map(|_| Utc::now() + Duration::hours(24));

        sqlx::query_as::<_, RecurringBookingSeries>(
            r#"
            INSERT INTO recurring_booking_series (
                id, organization_id, customer_id, walker_id, service_id, location_id,
                frequency, day_of_week, time_of_day, timezone, end_date, total_occurrences,
                price_cents_per_booking, default_notes, idempotency_key, idempotency_expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING id, organization_id, customer_id, walker_id, service_id, location_id,
                      frequency, day_of_week, time_of_day, timezone, end_date, total_occurrences,
                      is_active, price_cents_per_booking, default_notes,
                      idempotency_key, idempotency_expires_at, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(input.organization_id.as_uuid())
        .bind(input.customer_id.as_uuid())
        .bind(input.walker_id.as_uuid())
        .bind(input.service_id.as_uuid())
        .bind(input.location_id.as_uuid())
        .bind(&input.frequency)
        .bind(input.day_of_week)
        .bind(input.time_of_day)
        .bind(&input.timezone)
        .bind(input.end_date)
        .bind(input.total_occurrences)
        .bind(input.price_cents_per_booking)
        .bind(&input.default_notes)
        .bind(input.idempotency_key)
        .bind(idempotency_expires_at)
        .fetch_one(&mut **tx)
        .await
    }

    /// Find an existing series by idempotency key (for duplicate detection)
    pub async fn find_by_idempotency_key(
        pool: &PgPool,
        org_id: OrganizationId,
        idempotency_key: Uuid,
    ) -> Result<Option<RecurringBookingSeries>, sqlx::Error> {
        sqlx::query_as::<_, RecurringBookingSeries>(
            r#"
            SELECT id, organization_id, customer_id, walker_id, service_id, location_id,
                   frequency, day_of_week, time_of_day, timezone, end_date, total_occurrences,
                   is_active, price_cents_per_booking, default_notes,
                   idempotency_key, idempotency_expires_at, created_at, updated_at
            FROM recurring_booking_series
            WHERE idempotency_key = $1
              AND organization_id = $2
              AND idempotency_expires_at > NOW()
            "#,
        )
        .bind(idempotency_key)
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Find a recurring booking series by ID
    pub async fn find_by_id(
        pool: &PgPool,
        org_id: OrganizationId,
        id: RecurringBookingSeriesId,
    ) -> Result<Option<RecurringBookingSeries>, sqlx::Error> {
        sqlx::query_as::<_, RecurringBookingSeries>(
            r#"
            SELECT id, organization_id, customer_id, walker_id, service_id, location_id,
                   frequency, day_of_week, time_of_day, timezone, end_date, total_occurrences,
                   is_active, price_cents_per_booking, default_notes,
                   idempotency_key, idempotency_expires_at, created_at, updated_at
            FROM recurring_booking_series
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// Find all recurring series for a customer
    pub async fn find_by_customer(
        pool: &PgPool,
        org_id: OrganizationId,
        customer_id: UserId,
    ) -> Result<Vec<RecurringBookingSeries>, sqlx::Error> {
        sqlx::query_as::<_, RecurringBookingSeries>(
            r#"
            SELECT id, organization_id, customer_id, walker_id, service_id, location_id,
                   frequency, day_of_week, time_of_day, timezone, end_date, total_occurrences,
                   is_active, price_cents_per_booking, default_notes,
                   idempotency_key, idempotency_expires_at, created_at, updated_at
            FROM recurring_booking_series
            WHERE customer_id = $1 AND organization_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(customer_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    /// Find active recurring series for a customer
    pub async fn find_active_by_customer(
        pool: &PgPool,
        org_id: OrganizationId,
        customer_id: UserId,
    ) -> Result<Vec<RecurringBookingSeries>, sqlx::Error> {
        sqlx::query_as::<_, RecurringBookingSeries>(
            r#"
            SELECT id, organization_id, customer_id, walker_id, service_id, location_id,
                   frequency, day_of_week, time_of_day, timezone, end_date, total_occurrences,
                   is_active, price_cents_per_booking, default_notes,
                   idempotency_key, idempotency_expires_at, created_at, updated_at
            FROM recurring_booking_series
            WHERE customer_id = $1 AND organization_id = $2 AND is_active = true
            ORDER BY created_at DESC
            "#,
        )
        .bind(customer_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    /// Find all recurring series for a walker
    pub async fn find_by_walker(
        pool: &PgPool,
        org_id: OrganizationId,
        walker_id: UserId,
    ) -> Result<Vec<RecurringBookingSeries>, sqlx::Error> {
        sqlx::query_as::<_, RecurringBookingSeries>(
            r#"
            SELECT id, organization_id, customer_id, walker_id, service_id, location_id,
                   frequency, day_of_week, time_of_day, timezone, end_date, total_occurrences,
                   is_active, price_cents_per_booking, default_notes,
                   idempotency_key, idempotency_expires_at, created_at, updated_at
            FROM recurring_booking_series
            WHERE walker_id = $1 AND organization_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(walker_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    /// Deactivate a recurring series
    pub async fn deactivate(
        pool: &PgPool,
        org_id: OrganizationId,
        id: RecurringBookingSeriesId,
    ) -> Result<Option<RecurringBookingSeries>, sqlx::Error> {
        sqlx::query_as::<_, RecurringBookingSeries>(
            r#"
            UPDATE recurring_booking_series
            SET is_active = false, updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, customer_id, walker_id, service_id, location_id,
                      frequency, day_of_week, time_of_day, timezone, end_date, total_occurrences,
                      is_active, price_cents_per_booking, default_notes,
                      idempotency_key, idempotency_expires_at, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    /// List all recurring series for an organization
    pub async fn list_all(
        pool: &PgPool,
        org_id: OrganizationId,
    ) -> Result<Vec<RecurringBookingSeries>, sqlx::Error> {
        sqlx::query_as::<_, RecurringBookingSeries>(
            r#"
            SELECT id, organization_id, customer_id, walker_id, service_id, location_id,
                   frequency, day_of_week, time_of_day, timezone, end_date, total_occurrences,
                   is_active, price_cents_per_booking, default_notes,
                   idempotency_key, idempotency_expires_at, created_at, updated_at
            FROM recurring_booking_series
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_all(pool)
        .await
    }

    /// Count active recurring series for an organization
    pub async fn count_active(pool: &PgPool, org_id: OrganizationId) -> Result<i64, sqlx::Error> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM recurring_booking_series
            WHERE organization_id = $1 AND is_active = true
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_one(pool)
        .await?;

        Ok(result.0)
    }
}

/// Generate occurrence dates for a recurring series
pub fn generate_occurrence_dates(
    start_date: NaiveDate,
    frequency: RecurrenceFrequency,
    day_of_week: i32,
    end_date: Option<NaiveDate>,
    total_occurrences: Option<i32>,
) -> Vec<NaiveDate> {
    let mut dates = Vec::new();
    let max_occurrences = total_occurrences.unwrap_or(52) as usize; // Default to 1 year of weekly
    let end = end_date.unwrap_or(start_date + Duration::days(365));

    // Find the first occurrence on the correct day of week
    let target_weekday = match day_of_week {
        0 => Weekday::Sun,
        1 => Weekday::Mon,
        2 => Weekday::Tue,
        3 => Weekday::Wed,
        4 => Weekday::Thu,
        5 => Weekday::Fri,
        6 => Weekday::Sat,
        _ => Weekday::Mon,
    };

    let mut current = start_date;
    while current.weekday() != target_weekday {
        current = current + Duration::days(1);
    }

    while dates.len() < max_occurrences && current <= end {
        dates.push(current);

        current = match frequency {
            RecurrenceFrequency::Weekly => current + Duration::weeks(1),
            RecurrenceFrequency::BiWeekly => current + Duration::weeks(2),
            RecurrenceFrequency::Monthly => {
                // Add one month - find the same weekday in the next month
                let next_month = if current.month() == 12 {
                    NaiveDate::from_ymd_opt(current.year() + 1, 1, 1)
                } else {
                    NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1)
                };

                if let Some(nm) = next_month {
                    // Find the same weekday in the next month
                    let week_of_month = (current.day() - 1) / 7;
                    let mut candidate = nm;
                    while candidate.weekday() != target_weekday {
                        candidate = candidate + Duration::days(1);
                    }
                    candidate = candidate + Duration::weeks(week_of_month as i64);

                    // If we went past the month, use the last occurrence
                    if candidate.month() != nm.month() {
                        candidate = candidate - Duration::weeks(1);
                    }
                    candidate
                } else {
                    break;
                }
            }
        };
    }

    dates
}

/// Convert a date and time to UTC datetime
pub fn to_utc_datetime(date: NaiveDate, time: NaiveTime, timezone: &str) -> Option<DateTime<Utc>> {
    let tz: Tz = timezone.parse().ok()?;
    let naive_dt = date.and_time(time);
    tz.from_local_datetime(&naive_dt)
        .single()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Check for conflicts with existing bookings (legacy N-query approach)
/// Deprecated: Use check_conflicts_batch for better performance
pub async fn check_conflicts(
    pool: &PgPool,
    org_id: OrganizationId,
    walker_id: UserId,
    dates: &[NaiveDate],
    time_of_day: NaiveTime,
    duration_minutes: i32,
    timezone: &str,
) -> Result<Vec<OccurrenceConflict>, sqlx::Error> {
    // Delegate to batch implementation
    check_conflicts_batch(
        pool,
        org_id,
        walker_id,
        dates,
        time_of_day,
        duration_minutes,
        timezone,
    )
    .await
}

/// Batch conflict detection - checks all dates in 2 queries (bookings + blocks)
pub async fn check_conflicts_batch(
    pool: &PgPool,
    org_id: OrganizationId,
    walker_id: UserId,
    dates: &[NaiveDate],
    time_of_day: NaiveTime,
    duration_minutes: i32,
    timezone: &str,
) -> Result<Vec<OccurrenceConflict>, sqlx::Error> {
    if dates.is_empty() {
        return Ok(Vec::new());
    }

    // Pre-compute all UTC start/end times
    let mut time_windows: Vec<(NaiveDate, DateTime<Utc>, DateTime<Utc>)> =
        Vec::with_capacity(dates.len());
    let mut conflicts = Vec::new();

    for date in dates {
        match to_utc_datetime(*date, time_of_day, timezone) {
            Some(start) => {
                let end = start + Duration::minutes(duration_minutes as i64);
                time_windows.push((*date, start, end));
            }
            None => {
                conflicts.push(OccurrenceConflict {
                    date: *date,
                    reason: "Invalid timezone conversion".to_string(),
                });
            }
        }
    }

    if time_windows.is_empty() {
        return Ok(conflicts);
    }

    // Get the overall date range for the batch query
    let min_start = time_windows.iter().map(|(_, s, _)| s).min().unwrap();
    let max_end = time_windows.iter().map(|(_, _, e)| e).max().unwrap();

    // Fetch all potentially conflicting bookings in one query
    #[derive(sqlx::FromRow)]
    struct ConflictingBooking {
        scheduled_start: DateTime<Utc>,
        scheduled_end: DateTime<Utc>,
    }

    let existing_bookings: Vec<ConflictingBooking> = sqlx::query_as(
        r#"
        SELECT scheduled_start, scheduled_end
        FROM bookings
        WHERE walker_id = $1
          AND organization_id = $2
          AND status NOT IN ('cancelled', 'completed')
          AND scheduled_start < $4
          AND scheduled_end > $3
        ORDER BY scheduled_start
        "#,
    )
    .bind(walker_id.as_uuid())
    .bind(org_id.as_uuid())
    .bind(min_start)
    .bind(max_end)
    .fetch_all(pool)
    .await?;

    // Fetch all potentially conflicting blocks in one query
    #[derive(sqlx::FromRow)]
    struct ConflictingBlock {
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    }

    let existing_blocks: Vec<ConflictingBlock> = sqlx::query_as(
        r#"
        SELECT start_time, end_time
        FROM blocks
        WHERE walker_id = $1
          AND organization_id = $2
          AND start_time < $4
          AND end_time > $3
        ORDER BY start_time
        "#,
    )
    .bind(walker_id.as_uuid())
    .bind(org_id.as_uuid())
    .bind(min_start)
    .bind(max_end)
    .fetch_all(pool)
    .await?;

    // Check each occurrence against fetched conflicts (in-memory filtering)
    for (date, start, end) in time_windows {
        // Check booking conflicts
        let has_booking_conflict = existing_bookings
            .iter()
            .any(|b| b.scheduled_start < end && b.scheduled_end > start);

        if has_booking_conflict {
            conflicts.push(OccurrenceConflict {
                date,
                reason: "Walker has conflicting booking".to_string(),
            });
            continue;
        }

        // Check block conflicts
        let has_block_conflict = existing_blocks
            .iter()
            .any(|b| b.start_time < end && b.end_time > start);

        if has_block_conflict {
            conflicts.push(OccurrenceConflict {
                date,
                reason: "Walker has blocked time".to_string(),
            });
        }
    }

    Ok(conflicts)
}
