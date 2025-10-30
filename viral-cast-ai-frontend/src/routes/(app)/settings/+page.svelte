<script lang="ts">
  import type { PageData } from './$types';
  let { data }: { data: PageData } = $props();

  import type { Province, Regency, District, Village } from '$lib/types/regions';
  import { getProvincesApi, getRegenciesApi, getDistrictsApi, getVillagesApi } from '$lib/services/regions.js';
  import { t } from '$lib/stores/i18n';
  import { uploadUserProfilePhotoApi, uploadUserBackgroundApi, patchProfileApi } from '$lib/services/profile.js'
  import { uploadStoreImageApi, patchStoreApi } from '$lib/services/store.js'

  // Helper to resolve image URLs with API base
  function resolveImageUrl(u?: string): string {
    if (!u || typeof u !== 'string') return '';
    const trimmed = String(u).trim().replace(/^`+|`+$/g, '');
    if (/^https?:\/\//i.test(trimmed)) return trimmed;
    const base = 'http://localhost:12000';
    const path = trimmed.startsWith('/') ? trimmed : `/${trimmed}`;
    return `${base}${path}`;
  }

  // Prefer backend profile data if available, fallback to auth user locals
  const rawProfile: any = data?.profile ?? null;
  const rawUserAuth: any = data?.user ?? {};
  let user = $state({
    name: (rawProfile?.name ?? rawUserAuth?.name ?? '—'),
    title: rawUserAuth?.title ?? '',
    avatar: resolveImageUrl(
      rawProfile?.photo_profile ??
      rawUserAuth?.avatar ??
      '/images/user.png'
    )
  });
  let backgroundUrl = $state(resolveImageUrl(data?.profile?.background_profile || '/images/banner.jpg'));
  const storeUuid: string = (data?.profile?.store_uuid ?? data?.store?.store?.uuid ?? data?.store?.uuid ?? '') as string;
  let brandUrl = $state(resolveImageUrl(data?.store?.store?.brand_url ?? data?.store?.brand_url ?? '/images/brand.png'));

  // Profiles API form state
  let first_name = $state(rawProfile?.first_name ?? '');
  let last_name = $state(rawProfile?.last_name ?? '');
  let gender = $state<'MALE' | 'FEMALE' | 'OTHER'>(rawProfile?.gender ?? 'MALE');
  let telp = $state(rawProfile?.telp ?? '');
  let birth_date = $state(rawProfile?.birth_date ?? '');
  let birth_place = $state(rawProfile?.birth_place ?? '');
  let roles_number = $state<number | ''>(typeof rawProfile?.roles_number === 'number' ? rawProfile.roles_number : '');
  let store_uuid = $state(rawProfile?.store_uuid ?? '');
  let province_code = $state(rawProfile?.province_code ?? '');
  let regency_code = $state(rawProfile?.regency_code ?? '');
  let district_code = $state(rawProfile?.district_code ?? '');
  let village_code = $state(rawProfile?.village_code ?? '');
  let rt = $state(rawProfile?.rt ?? '');
  let rw = $state(rawProfile?.rw ?? '');
  let postal_code = $state(rawProfile?.postal_code ?? '');

  // Derive first/last name from full name if not present
  $effect(() => {
    const base = (rawProfile?.name || user.name || '').trim();
    const parts = base ? base.split(/\s+/) : [];
    if (!first_name && parts.length) first_name = parts[0] || '';
    if (!last_name && parts.length > 1) last_name = parts.slice(1).join(' ') || '';
  });

  // Avatar upload helpers
  let avatarFormEl: HTMLFormElement | null = null;
  let avatarInputEl: HTMLInputElement | null = null;
  function submitAvatar() {
    avatarFormEl?.requestSubmit?.();
  }
  async function handleAvatarChange(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input?.files?.[0];
    if (!file) return;
    const localPreview = URL.createObjectURL(file);
    try {
      // Tampilkan preview instan tanpa reload
      user.avatar = localPreview;
      const res = await uploadUserProfilePhotoApi(file);
      const imageUrl = (res as any)?.data?.image_url || null;
      if (imageUrl) {
        await patchProfileApi({ photo_profile: imageUrl });
        // Pakai URL absolut supaya tidak fallback ke default
        user.avatar = resolveImageUrl(imageUrl);
        URL.revokeObjectURL(localPreview);
      }
    } catch (err) {
      console.error('Failed to upload avatar', err);
    }
  }

  // Background cover upload helpers
  let bgFormEl: HTMLFormElement | null = null;
  let bgInputEl: HTMLInputElement | null = null;
  function submitBackground() {
    bgFormEl?.requestSubmit?.();
  }
  async function handleBackgroundChange(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input?.files?.[0];
    if (!file) return;
    const localPreview = URL.createObjectURL(file);
    try {
      // Preview cover instan
      backgroundUrl = localPreview;
      const res = await uploadUserBackgroundApi(file);
      const imageUrl = (res as any)?.data?.image_url || null;
      if (imageUrl) {
        await patchProfileApi({ background_profile: imageUrl });
        backgroundUrl = resolveImageUrl(imageUrl);
        URL.revokeObjectURL(localPreview);
      }
    } catch (err) {
      console.error('Failed to upload cover image', err);
    }
  }

  // Store brand upload helpers
  let brandFormEl: HTMLFormElement | null = null;
  let brandInputEl: HTMLInputElement | null = null;
  function submitBrand() { brandFormEl?.requestSubmit?.(); }
  async function handleBrandChange(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input?.files?.[0];
    if (!file || !storeUuid) return;
    try {
      const res = await uploadStoreImageApi(file);
      const imageUrl = (res as any)?.data?.image_url || null;
      if (imageUrl) {
        await patchStoreApi(storeUuid, { brand_url: imageUrl } as any);
        brandUrl = resolveImageUrl(imageUrl);
      }
    } catch (err) {
      console.error('Failed to upload brand image', err);
    }
  }

  // Regions cascaded selects state
  let provinces = $state<Province[]>([]);
  let regencies = $state<Regency[]>([]);
  let districts = $state<District[]>([]);
  let villages = $state<Village[]>([]);

  // Toggle edit mode: view-only by default
  let isEditing = $state(false);

  // Track previous codes to avoid resetting on initial load
  let prevProvinceCode = $state(province_code);
  let prevRegencyCode = $state(regency_code);
  let prevDistrictCode = $state(district_code);

  function formatRegionName(name?: string) {
    if (!name) return '';
    const parts = String(name).trim().split(/\s+/);
    return parts
      .map((w) => {
        if (/^[A-Z0-9]+$/.test(w) && w.length <= 3) return w; // keep acronyms (DKI, DIY, II, IV)
        const first = w.charAt(0).toUpperCase();
        const rest = w.slice(1).toLowerCase();
        return first + rest;
      })
      .join(' ');
  }

  // Optional search terms (simple client-side)
  // Search inputs dihapus; dropdown dibuat standar tanpa pencarian
   let edit = $state({
     first_name: false,
     last_name: false,
     gender: false,
     telp: false,
     birth_date: false,
     birth_place: false,
     rt: false,
     rw: false,
     postal_code: false
   });
  let editRegion = $state(false);

  async function loadProvinces(search?: string) {
    try {
      const res = await getProvincesApi({ limit: 100, search });
      provinces = res.data || [];
    } catch (e) {
      provinces = [];
    }
  }

  async function loadRegenciesForProvince(codeOrUuid: string, search?: string) {
    if (!codeOrUuid) { regencies = []; return; }
    try {
      const res = await getRegenciesApi({ province_code: codeOrUuid, search });
      regencies = res.data || [];
    } catch (e) {
      regencies = [];
    }
  }

  async function loadDistrictsForRegency(codeOrUuid: string, search?: string) {
    if (!codeOrUuid) { districts = []; return; }
    try {
      const res = await getDistrictsApi({ regency_code: codeOrUuid, search });
      districts = res.data || [];
    } catch (e) {
      districts = [];
    }
  }

  async function loadVillagesForDistrict(codeOrUuid: string, search?: string) {
    if (!codeOrUuid) { villages = []; return; }
    try {
      const res = await getVillagesApi({ district_code: codeOrUuid, search });
      villages = res.data || [];
    } catch (e) {
      villages = [];
    }
  }

  async function prefillRegionsFromVillageCode() {
    try {
      const resV = await getVillagesApi({ code: String(village_code), limit: 1 });
      const vMatch = (resV.data || []).find((v) => v.code === village_code) ?? resV.data?.[0];
      if (!vMatch) return;

      // Prefill codes directly from village record
      if (!province_code && vMatch.province_code) {
        province_code = vMatch.province_code;
      }

      if (!regency_code && vMatch.regency_code) {
        regency_code = vMatch.regency_code;
      }

      if (!district_code && vMatch.district_code) {
        district_code = vMatch.district_code;
      }
    } catch (e) {
      // ignore prefill errors
    }
  }

  // Initial load: provinces, and drill down if codes already present
  $effect(() => { loadProvinces(); });

  $effect(() => {
    if (province_code) {
      loadRegenciesForProvince(province_code);
    } else {
      regencies = [];
    }
    // Only reset children when user actually changes province in edit mode
    if (isEditing && province_code !== prevProvinceCode) {
      regency_code = '';
      district_code = '';
      village_code = '';
    }
    prevProvinceCode = province_code;
  });

  $effect(() => {
    if (regency_code) {
      loadDistrictsForRegency(regency_code);
    } else {
      districts = [];
    }
    // Only reset children when user actually changes regency in edit mode
    if (isEditing && regency_code !== prevRegencyCode) {
      district_code = '';
      village_code = '';
    }
    prevRegencyCode = regency_code;
  });

  $effect(() => {
    if (district_code) {
      loadVillagesForDistrict(district_code);
    } else {
      villages = [];
    }
    // Only reset children when user actually changes district in edit mode
    if (isEditing && district_code !== prevDistrictCode) {
      village_code = '';
    }
    prevDistrictCode = district_code;
  });

  // Prefill cascade from village_code on initial load if other codes missing
  $effect(() => {
    if (village_code && (!province_code || !regency_code || !district_code)) {
      prefillRegionsFromVillageCode();
    }
  });

</script>

<!-- PAGE WRAPPER -->
<section class="space-y-6 px-4 py-6">
	<header class="flex items-center justify-between">
        <div>
            <h1 class="text-xl font-semibold">{t('profile_title')}</h1>
            <p class="text-sm opacity-70">{t('profile_settings')}</p>
        </div>
    </header>

	<!-- PROFILE HEADER CARD -->
	<div
		class="shadow-card relative overflow-hidden rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)]"
	>
        <img src={backgroundUrl} alt={t('background')} class="absolute inset-0 h-full w-full object-cover opacity-30" onerror={(e) => ((e.currentTarget as HTMLImageElement).src = '/images/banner.jpg')} />

        <!-- Cover upload button -->
        <form method="post" action="?/upload_background" enctype="multipart/form-data" bind:this={bgFormEl} class="absolute right-4 top-4 z-20">
          <input type="file" name="background" accept="image/*" class="hidden" bind:this={bgInputEl} onchange={handleBackgroundChange} />
          <button type="button" class="rounded-[var(--radius-pill)] border border-[var(--ui-border)] bg-black/20 px-3 py-1.5 text-xs text-white backdrop-blur hover:bg-black/30" onclick={() => bgInputEl?.click()}>
            {t('change_cover')}
          </button>
        </form>

		<div class="relative z-10 grid gap-6 p-6 md:grid-cols-[auto_1fr_auto] md:items-center">
            <!-- Avatar with overlay upload -->
            <form method="post" action="?/upload_avatar" enctype="multipart/form-data" bind:this={avatarFormEl}>
              <div class="group relative h-24 w-24">
                <img
                  src={user.avatar}
                  alt={t('user_avatar')}
                  width="96"
                  height="96"
                  class="h-24 w-24 rounded-full object-cover ring-4 ring-[color-mix(in_oklch,var(--color-surface),black_6%)]"
                  onerror={(e) => ((e.currentTarget as HTMLImageElement).src = '/images/user.png')}
                />
                <input type="file" name="avatar" accept="image/*" class="hidden" bind:this={avatarInputEl} onchange={handleAvatarChange} />
                <button type="button" class="absolute inset-0 hidden items-center justify-center rounded-full bg-black/40 text-xs text-white group-hover:flex" onclick={() => avatarInputEl?.click()}>
                  {t('change_photo')}
                </button>
              </div>
            </form>

            <!-- Name from first/last and gender -->
            <div>
                <h2 class="text-lg font-semibold">{first_name} {last_name}</h2>
                <div class="mt-1 text-sm opacity-80">{gender}</div>
                {#if data?.regionNames && (data.regionNames.province || data.regionNames.regency || data.regionNames.district || data.regionNames.village)}
                   <div class="mt-2 text-xs opacity-80">
                     {t('region')}: {formatRegionName(data.regionNames.province) || '—'} / {formatRegionName(data.regionNames.regency) || '—'} / {formatRegionName(data.regionNames.district) || '—'} / {formatRegionName(data.regionNames.village) || '—'}
                   </div>
                 {/if}
                {#if data?.store}
                  <div class="mt-2 text-xs opacity-80">{t('store')}: {data.store?.store?.name ?? data.store?.name ?? '—'}</div>
                {/if}
            </div>

            <!-- No edit button: editing via form below -->

        </div>
	</div>

    <!-- PROFILE FORM (All fields from Profiles API) -->
    <section class="shadow-card rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-6">
      <h3 class="text-base font-semibold">{t('profile_title')}</h3>
      <p class="mt-1 mb-4 text-sm opacity-70">{t('profile_settings')}</p>
      <div class="mb-4 flex justify-end">
        <button type="button" class={isEditing ? 'rounded-[var(--radius-pill)] px-3 py-1.5 text-sm bg-[var(--color-yellow-deep-600)] text-white' : 'rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-3 py-1.5 text-sm'} onclick={() => (isEditing = !isEditing)}>{isEditing ? t('cancel') : t('edit')}</button>
      </div>

      <form method="post" action="?/update_profile" class="grid gap-4 md:grid-cols-2">
        <div>
          <label for="first_name" class="block text-sm mb-1">{t('first_name')}</label>
          <input id="first_name" name="first_name" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" bind:value={first_name} disabled={!isEditing} />
        </div>
        <div>
          <label for="last_name" class="block text-sm mb-1">{t('last_name')}</label>
          <input id="last_name" name="last_name" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" bind:value={last_name} disabled={!isEditing} />
        </div>
        <div>
          <label for="gender" class="block text-sm mb-1">{t('gender')}</label>
          <select id="gender" name="gender" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" bind:value={gender} disabled={!isEditing}>
            <option value="MALE">{t('male')}</option>
            <option value="FEMALE">{t('female')}</option>
            <option value="OTHER">{t('other')}</option>
          </select>
        </div>
        <div>
          <label for="telp" class="block text-sm mb-1">{t('phone')}</label>
          <input id="telp" name="telp" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" bind:value={telp} disabled={!isEditing} />
        </div>
        <div>
          <label for="birth_date" class="block text-sm mb-1">{t('birth_date')}</label>
          <input id="birth_date" name="birth_date" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="date" bind:value={birth_date} disabled={!isEditing} />
        </div>
        <div>
          <label for="birth_place" class="block text-sm mb-1">{t('birth_place')}</label>
          <input id="birth_place" name="birth_place" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" bind:value={birth_place} disabled={!isEditing} />
        </div>

        <div>
          <div class="flex items-center justify-between">
            <label for="province_code" class="block text-sm mb-1">{t('province')}</label>
          </div>
          <select id="province_code" name="province_code" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition" bind:value={province_code} disabled={!isEditing}>
            <option value="" disabled>{t('select_province')}</option>
            {#each provinces as p}
              <option value={p.code}>{formatRegionName(p.name)}</option>
            {/each}
          </select>
        </div>
        <div>
          <div class="flex items-center justify-between">
            <label for="regency_code" class="block text-sm mb-1">{t('regency_city')}</label>
          </div>
          <select id="regency_code" name="regency_code" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition" bind:value={regency_code} disabled={!isEditing || !provinces.length || !province_code}>
            <option value="" disabled>{t('select_regency_city')}</option>
            {#each regencies as r}
              <option value={r.code}>{formatRegionName(r.name)}</option>
            {/each}
          </select>
        </div>
        <div>
          <div class="flex items-center justify-between">
            <label for="district_code" class="block text-sm mb-1">{t('district')}</label>
          </div>
          <select id="district_code" name="district_code" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition" bind:value={district_code} disabled={!isEditing || !regencies.length || !regency_code}>
            <option value="" disabled>{t('select_district')}</option>
            {#each districts as d}
              <option value={d.code}>{formatRegionName(d.name)}</option>
            {/each}
          </select>
        </div>
        <div>
          <div class="flex items-center justify-between">
            <label for="village_code" class="block text-sm mb-1">{t('village_subdistrict')}</label>
          </div>
          <select id="village_code" name="village_code" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition" bind:value={village_code} disabled={!isEditing || !districts.length || !district_code}>
            <option value="" disabled>{t('select_village_subdistrict')}</option>
            {#each villages as v}
              <option value={v.code}>{formatRegionName(v.name)}</option>
            {/each}
          </select>
        </div>

        <!-- Region name inputs removed as requested -->

        <div>
          <label for="rt" class="block text-sm mb-1">{t('rt')}</label>
          <input id="rt" name="rt" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" bind:value={rt} disabled={!isEditing} />
        </div>
        <div>
          <label for="rw" class="block text-sm mb-1">{t('rw')}</label>
          <input id="rw" name="rw" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" bind:value={rw} disabled={!isEditing} />
        </div>
        <div class="md:col-span-2">
          <label for="postal_code" class="block text-sm mb-1">{t('postal_code')}</label>
          <input id="postal_code" name="postal_code" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" bind:value={postal_code} disabled={!isEditing} />
        </div>

        <div class="md:col-span-2 mt-2 flex gap-2">
          <button type="submit" class="rounded-[var(--radius-pill)] bg-[--nav-active-text] px-4 py-2 text-sm text-white disabled:opacity-50 disabled:cursor-not-allowed" style="background: var(--nav-active-bg-solid);" disabled={!isEditing}>{t('save_profile')}</button>
        </div>
      </form>
    </section>

    <section class="shadow-card rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-6 mt-6">
      <div class="flex items-center gap-3">
        {#if data?.store}
          <form method="post" action="?/upload_store_brand" enctype="multipart/form-data" bind:this={brandFormEl} class="relative group">
            <div class="relative h-10 md:h-12">
              <img class="h-10 md:h-12 object-contain rounded" src={brandUrl} alt={t('brand')} onerror={(e) => ((e.currentTarget as HTMLImageElement).src = '/images/brand.png')} />
              <input type="file" name="brand" accept="image/*" class="hidden" bind:this={brandInputEl} onchange={handleBrandChange} />
              <button type="button" class="absolute inset-0 hidden items-center justify-center rounded bg-black/40 text-xs text-white group-hover:flex" onclick={() => brandInputEl?.click()}>
                {t('change_photo')}
              </button>
            </div>
          </form>
        {/if}
        <h3 class="text-base font-semibold">{t('store_title')}</h3>
      </div>
      {#if data?.store}
        <div class="mt-3 grid gap-4 md:grid-cols-2">
          <div>
            <label class="block text-sm mb-1">{t('store_name')}</label>
            <input class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" value={(data.store?.store?.name ?? data.store?.name) || ''} disabled />
          </div>
          <div>
            <label class="block text-sm mb-1">{t('phone')}</label>
            <input class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" value={data.store?.store?.telp ?? data.store?.telp ?? '—'} disabled />
          </div>
          <div>
            <label class="block text-sm mb-1">{t('whatsapp')}</label>
            <input class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" value={data.store?.store?.whatsapp ?? data.store?.whatsapp ?? '—'} disabled />
          </div>
          <div>
            <label class="block text-sm mb-1">{t('instagram')}</label>
            <input class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" value={data.store?.store?.instagram ?? data.store?.instagram ?? '—'} disabled />
          </div>

          <!-- Urutan wilayah seperti di Profil: Provinsi → Kabupaten/Kota → Kecamatan → Desa/Kelurahan -->
          <div>
            <label class="block text-sm mb-1">{t('province')}</label>
            <input class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" value={formatRegionName(data.storeRegionNames?.province) || '—'} disabled />
          </div>
          <div>
            <label class="block text-sm mb-1">{t('regency_city')}</label>
            <input class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" value={formatRegionName(data.storeRegionNames?.regency) || '—'} disabled />
          </div>
          <div>
            <label class="block text-sm mb-1">{t('district')}</label>
            <input class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" value={formatRegionName(data.storeRegionNames?.district) || '—'} disabled />
          </div>
          <div>
            <label class="block text-sm mb-1">{t('village_subdistrict')}</label>
            <input class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" value={formatRegionName(data.storeRegionNames?.village) || '—'} disabled />
          </div>

          <!-- Setelah Desa → RT → RW → Kode Pos -->
          <div>
            <label class="block text-sm mb-1">{t('rt')}</label>
            <input class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" value={data.store?.store?.rt ?? data.store?.rt ?? '—'} disabled />
          </div>
          <div>
            <label class="block text-sm mb-1">{t('rw')}</label>
            <input class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" value={data.store?.store?.rw ?? data.store?.rw ?? '—'} disabled />
          </div>
          <div>
            <label class="block text-sm mb-1">{t('postal_code')}</label>
            <input class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-white dark:bg-[color-mix(in_oklch,var(--color-surface),white_12%)] disabled:bg-gray-100 dark:disabled:bg-[color-mix(in_oklch,var(--color-surface),black_20%)] text-black dark:text-white disabled:text-gray-500 dark:disabled:text-gray-400 outline-none shadow-sm focus:ring-2 focus:ring-[var(--nav-active-bg-solid)] focus:border-transparent transition disabled:cursor-not-allowed" type="text" value={data.store?.store?.postal_code ?? data.store?.postal_code ?? '—'} disabled />
          </div>

          <!-- Brand image moved to header (top-left) -->
        </div>
      {:else}
        <div class="mt-3 text-sm opacity-70">{t('no_store_data')}</div>
      {/if}
    </section>

    <!-- Focus only on Profiles API; remove legacy modals, address & store blocks -->
</section>
