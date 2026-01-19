use axum::Json;
use db::repositories::{BookingRepository, PaymentRepository};
use serde::Serialize;

use crate::{auth::TenantContext, error::ApiResult};

#[derive(Debug, Serialize)]
pub struct DashboardMetricsResponse {
    /// Number of bookings scheduled for today
    pub today_booking_count: i64,
    /// Total revenue for the current week in cents
    pub week_revenue_cents: i64,
    /// Number of bookings with pending status
    pub pending_booking_count: i64,
}

/// GET /admin/dashboard/metrics - Fetch dashboard metrics for tenant
pub async fn get_metrics(tenant: TenantContext) -> ApiResult<Json<DashboardMetricsResponse>> {
    // Fetch all metrics in parallel for better performance
    let (today_count, pending_count, week_revenue) = tokio::try_join!(
        BookingRepository::count_today(&tenant.pool, tenant.org_id),
        BookingRepository::count_pending(&tenant.pool, tenant.org_id),
        PaymentRepository::revenue_this_week(&tenant.pool, tenant.org_id),
    )?;

    Ok(Json(DashboardMetricsResponse {
        today_booking_count: today_count,
        week_revenue_cents: week_revenue,
        pending_booking_count: pending_count,
    }))
}
