export type Gender = 'MALE' | 'FEMALE' | 'OTHER';

export interface Profile {
  uuid?: string;
  name: string;
  photo_profile?: string | null;
  background_profile?: string | null;
  gender: Gender;
  telp?: string | null;
  birth_date?: string | null; // ISO date string YYYY-MM-DD
  birth_place?: string | null;
  roles_number?: number;
  store_uuid?: string | null;
  store_name?: string | null;
  store_telp?: string | null;
  province_code?: string | null;
  regency_code?: string | null;
  district_code?: string | null;
  village_code?: string | null;
  rt?: string | null;
  rw?: string | null;
  postal_code?: string | null;
  created_at?: number;
  updated_at?: number;
}

export interface CreateProfileRequest {
  name?: string; // some endpoints may accept first/last instead of name
  first_name?: string;
  last_name?: string;
  photo_profile?: string | null;
  background_profile?: string | null;
  gender: Gender;
  telp?: string | null;
  birth_date?: string | null; // ISO date string
  birth_place?: string | null;
  roles_number?: number;
  store_uuid?: string | null;
  store_name?: string | null;
  store_telp?: string | null;
  province_code?: string | null;
  regency_code?: string | null;
  district_code?: string | null;
  village_code?: string | null;
  rt?: string | null;
  rw?: string | null;
  postal_code?: string | null;
}

export interface UpdateProfileRequest {
  name?: string;
  first_name?: string;
  last_name?: string;
  photo_profile?: string | null;
  background_profile?: string | null;
  gender?: Gender;
  telp?: string | null;
  birth_date?: string | null; // ISO date string
  birth_place?: string | null;
  roles_number?: number;
  store_uuid?: string | null;
  store_name?: string | null;
  store_telp?: string | null;
  province_code?: string | null;
  regency_code?: string | null;
  district_code?: string | null;
  village_code?: string | null;
  rt?: string | null;
  rw?: string | null;
  postal_code?: string | null;
}

export interface ProfileResponse {
  code: number;
  status: string;
  message: string;
  data: {
    profile: Profile;
  } | any; // fallback if backend returns different shape
  errors: any;
}