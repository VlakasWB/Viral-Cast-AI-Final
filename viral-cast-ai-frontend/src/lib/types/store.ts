export interface Store {
  uuid: string;
  name: string;
  brand_url?: string | null;
  telp?: string | null;
  whatsapp?: string | null;
  instagram?: string | null;
  postal_code?: string | null;
  rt?: string | null;
  rw?: string | null;
  province_code?: string | null;
  regency_code?: string | null;
  district_code?: string | null;
  village_code?: string | null;
  created_at?: number;
  updated_at?: number;
}

export interface CreateStoreRequest {
  name: string;
  brand_url?: string | null;
  telp?: string | null;
  whatsapp?: string | null;
  instagram?: string | null;
  postal_code?: string | null;
  rt?: string | null;
  rw?: string | null;
  province_code?: string | null;
  regency_code?: string | null;
  district_code?: string | null;
  village_code?: string | null;
}

export interface UpdateStoreRequest {
  name?: string;
  brand_url?: string | null;
  telp?: string | null;
  whatsapp?: string | null;
  instagram?: string | null;
  postal_code?: string | null;
  rt?: string | null;
  rw?: string | null;
  province_code?: string | null;
  regency_code?: string | null;
  district_code?: string | null;
  village_code?: string | null;
}

export interface StoreSingleResponse {
  code: number;
  status: string;
  message: string;
  data: { store: Store } | any;
  errors: any;
}