export interface Service {
  id: string;
  organization_id: string;
  name: string;
  description: string | null;
  duration_minutes: number;
  base_price_cents: number;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}
