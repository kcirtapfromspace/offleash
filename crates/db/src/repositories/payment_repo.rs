use shared::types::{BookingId, OrganizationId, PaymentId, UserId};
use sqlx::PgPool;

use crate::models::{CreatePayment, Payment, PaymentStatus};

pub struct PaymentRepository;

impl PaymentRepository {
    pub async fn create(pool: &PgPool, input: CreatePayment) -> Result<Payment, sqlx::Error> {
        let id = PaymentId::new();

        sqlx::query_as::<_, Payment>(
            r#"
            INSERT INTO payments (id, organization_id, booking_id, customer_id, amount_cents, payment_method)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, organization_id, booking_id, customer_id, amount_cents, status, square_payment_id, square_order_id, payment_method, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(input.organization_id.as_uuid())
        .bind(input.booking_id.as_uuid())
        .bind(input.customer_id.as_uuid())
        .bind(input.amount_cents)
        .bind(input.payment_method)
        .fetch_one(pool)
        .await
    }

    pub async fn find_by_id(
        pool: &PgPool,
        org_id: OrganizationId,
        id: PaymentId,
    ) -> Result<Option<Payment>, sqlx::Error> {
        sqlx::query_as::<_, Payment>(
            r#"
            SELECT id, organization_id, booking_id, customer_id, amount_cents, status, square_payment_id, square_order_id, payment_method, created_at, updated_at
            FROM payments
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_booking(
        pool: &PgPool,
        org_id: OrganizationId,
        booking_id: BookingId,
    ) -> Result<Option<Payment>, sqlx::Error> {
        sqlx::query_as::<_, Payment>(
            r#"
            SELECT id, organization_id, booking_id, customer_id, amount_cents, status, square_payment_id, square_order_id, payment_method, created_at, updated_at
            FROM payments
            WHERE booking_id = $1 AND organization_id = $2
            "#,
        )
        .bind(booking_id.as_uuid())
        .bind(org_id.as_uuid())
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_customer(
        pool: &PgPool,
        org_id: OrganizationId,
        customer_id: UserId,
    ) -> Result<Vec<Payment>, sqlx::Error> {
        sqlx::query_as::<_, Payment>(
            r#"
            SELECT id, organization_id, booking_id, customer_id, amount_cents, status, square_payment_id, square_order_id, payment_method, created_at, updated_at
            FROM payments
            WHERE customer_id = $1 AND organization_id = $2
            ORDER BY created_at DESC
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
        id: PaymentId,
        status: PaymentStatus,
        square_payment_id: Option<&str>,
        square_order_id: Option<&str>,
    ) -> Result<Option<Payment>, sqlx::Error> {
        sqlx::query_as::<_, Payment>(
            r#"
            UPDATE payments
            SET
                status = $3,
                square_payment_id = COALESCE($4, square_payment_id),
                square_order_id = COALESCE($5, square_order_id),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, booking_id, customer_id, amount_cents, status, square_payment_id, square_order_id, payment_method, created_at, updated_at
            "#,
        )
        .bind(id.as_uuid())
        .bind(org_id.as_uuid())
        .bind(status)
        .bind(square_payment_id)
        .bind(square_order_id)
        .fetch_optional(pool)
        .await
    }
}
