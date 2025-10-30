<script lang="ts">
	import Button from '$lib/components/ui/Button.svelte';
	import { page } from '$app/state';
	import {
		getProvincesApi,
		getRegenciesApi,
		getDistrictsApi,
		getVillagesApi
	} from '$lib/services/regions';
  import { getWeatherPredictionApi } from '$lib/services/weatherBmkg';
	import { t, locale } from '$lib/stores/i18n';
	import { get } from 'svelte/store';
	import { getMyProfileApi } from '$lib/services/profile';
	import { getStoreByUuidApi } from '$lib/services/store';

	// State dropdown berantai
	let provinces: { id: string; name: string }[] = $state([]);
	let regencies: { id: string; name: string }[] = $state([]);
	let districts: { id: string; name: string }[] = $state([]);
	let villages: { id: string; name: string }[] = $state([]);

	let provinceId = $state('');
  let regencyId = $state('');
  let districtId = $state('');
  let villageId = $state('');

  // Loading & error states
  let provincesLoading = $state(false);
  let regenciesLoading = $state(false);
  let districtsLoading = $state(false);
  let villagesLoading = $state(false);

  let provincesError = $state('');
  let regenciesError = $state('');
  let districtsError = $state('');
  let villagesError = $state('');

  async function loadProvinces() {
    provincesLoading = true;
    provincesError = '';
    try {
      const res = await getProvincesApi({ limit: 100 });
      provinces = res.data.map((p) => ({ id: p.uuid, name: p.name }));
    } catch (err) {
      provinces = [];
      provincesError = t('failed_provinces');
      console.error('Failed to load provinces', err);
    } finally {
      provincesLoading = false;
    }
  }

  async function onProvinceChange(id: string) {
    provinceId = id;
		// reset bawah
		regencyId = '';
		districtId = '';
		villageId = '';
		regencies = [];
		districts = [];
		villages = [];
    if (id) {
      regenciesLoading = true;
      regenciesError = '';
      try {
        const res = await getRegenciesApi({ province_uuid: id, limit: 100 });
        regencies = res.data.map((r) => ({ id: r.uuid, name: r.name }));
      } catch (err) {
        regencies = [];
        regenciesError = t('failed_regencies');
        console.error('Failed to load regencies', err);
      } finally {
        regenciesLoading = false;
      }
    }
	}

  async function onRegencyChange(id: string) {
    regencyId = id;
		districtId = '';
		villageId = '';
		districts = [];
		villages = [];
    if (id) {
      districtsLoading = true;
      districtsError = '';
      try {
        const res = await getDistrictsApi({ regency_uuid: id, limit: 100 });
        districts = res.data.map((d) => ({ id: d.uuid, name: d.name }));
      } catch (err) {
        districts = [];
        districtsError = t('failed_districts');
        console.error('Failed to load districts', err);
      } finally {
        districtsLoading = false;
      }
    }
	}

  async function onDistrictChange(id: string) {
    districtId = id;
		villageId = '';
		villages = [];
    if (id) {
      villagesLoading = true;
      villagesError = '';
      try {
        const res = await getVillagesApi({ district_uuid: id, limit: 100 });
        villages = res.data.map((v) => ({ id: v.uuid, name: v.name }));
      } catch (err) {
        villages = [];
        villagesError = t('failed_villages');
        console.error('Failed to load villages', err);
      } finally {
        villagesLoading = false;
      }
    }
	}

  // Prefill wilayah dari Profile → Store dan kunci dropdown
  async function loadRegionsFromProfileStore() {
    provincesLoading = regenciesLoading = districtsLoading = villagesLoading = true;
    provincesError = regenciesError = districtsError = villagesError = '';
    try {
      const profRes = await getMyProfileApi();
      const profile: any = profRes?.data?.profile ?? profRes?.data ?? {};
      const storeUuid = profile?.store_uuid;

      let store: any = null;
      if (storeUuid) {
        const storeRes = await getStoreByUuidApi(String(storeUuid));
        store = storeRes?.data?.store ?? storeRes?.data ?? null;
      }

      const province_code = store?.province_code ?? profile?.province_code ?? '';
      const regency_code = store?.regency_code ?? profile?.regency_code ?? '';
      const district_code = store?.district_code ?? profile?.district_code ?? '';
      const village_code = store?.village_code ?? profile?.village_code ?? '';

      provinceId = province_code || '';
      regencyId = regency_code || '';
      districtId = district_code || '';
      villageId = village_code || '';

      // Ambil nama berdasarkan code (masing-masing satu entri)
      if (provinceId) {
        try {
          const pRes = await getProvincesApi({ code: String(provinceId), limit: 1 });
          provinces = (pRes?.data || []).map((p: any) => ({ id: p.code, name: p.name }));
        } catch (e) {
          provinces = [];
          provincesError = t('failed_provinces');
        }
      } else {
        provinces = [];
      }

      if (regencyId) {
        try {
          const rRes = await getRegenciesApi({ code: String(regencyId), limit: 1 });
          regencies = (rRes?.data || []).map((r: any) => ({ id: r.code, name: r.name }));
        } catch (e) {
          regencies = [];
          regenciesError = t('failed_regencies');
        }
      } else {
        regencies = [];
      }

      if (districtId) {
        try {
          const dRes = await getDistrictsApi({ code: String(districtId), limit: 1 });
          districts = (dRes?.data || []).map((d: any) => ({ id: d.code, name: d.name }));
        } catch (e) {
          districts = [];
          districtsError = t('failed_districts');
        }
      } else {
        districts = [];
      }

      if (villageId) {
        try {
          const vRes = await getVillagesApi({ code: String(villageId), limit: 1 });
          villages = (vRes?.data || []).map((v: any) => ({ id: v.code, name: v.name }));
        } catch (e) {
          villages = [];
          villagesError = t('failed_villages');
        }
      } else {
        villages = [];
      }
    } catch (err) {
      console.error('Failed to load profile/store for weather', err);
      provincesError = provincesError || t('failed_provinces');
      regenciesError = regenciesError || t('failed_regencies');
      districtsError = districtsError || t('failed_districts');
      villagesError = villagesError || t('failed_villages');
    } finally {
      provincesLoading = regenciesLoading = districtsLoading = villagesLoading = false;
    }
  }


  // State & aksi untuk memuat prediksi cuaca BMKG (real API)
  let resultText = $state('');
  let weatherLoading = $state(false);
  let weatherError = $state('');
  let weather: any = $state(null);
  let expandedFlags: boolean[] = $state([]);

  function filterForecasts(list: any[] | undefined) {
    if (!Array.isArray(list) || list.length === 0) return [];

    const now = new Date();
    const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate());

    return list.filter((item) => {
      const date = parseDateSafe(item?.local_datetime || item?.datetime_utc);
      if (!date) return false;
      const dayDiff = Math.floor((date.getTime() - startOfToday.getTime()) / 86400000);
      return dayDiff >= 0 && dayDiff <= 3;
    });
  }

  async function showWeather() {
    weatherError = '';
    resultText = '';
    weather = null;
    if (!provinceId || !regencyId || !districtId || !villageId) {
      resultText = t('please_complete_selections');
      return;
    }
    weatherLoading = true;
    try {
      const region_code = villageId || districtId || regencyId || provinceId;
      const res = await getWeatherPredictionApi(region_code);
      const rawWeather = res?.data ?? null;
      const filteredForecasts = filterForecasts(rawWeather?.prakiraan_cuaca);
      weather = rawWeather
        ? { ...rawWeather, prakiraan_cuaca: filteredForecasts }
        : null;
      expandedFlags = filteredForecasts.map(() => false);
    } catch (err) {
      console.error('Failed to load weather prediction', err);
      weatherError = t('failed_prediction');
      } finally {
      weatherLoading = false;
    }
  }

  // ====== Formatting helpers for prediction display (English) ======
  const monthNames: Record<'en'|'id', string[]> = {
    en: ['January','February','March','April','May','June','July','August','September','October','November','December'],
    id: ['Januari','Februari','Maret','April','Mei','Juni','Juli','Agustus','September','Oktober','November','Desember']
  };

  function parseDateSafe(dt?: string): Date | null {
    if (!dt) return null;
    // Pastikan format kompatibel Date
    const normalized = dt.includes('T') ? dt : dt.replace(' ', 'T');
    const d = new Date(normalized);
    return isNaN(d.getTime()) ? null : d;
  }

  function getAmPm(hours: number): 'AM' | 'PM' {
    return hours < 12 ? 'AM' : 'PM';
  }

  function formatHour12(hours: number): string {
    const h = hours % 12 || 12;
    return String(h);
  }

  function looksNumeric(value?: string | number): boolean {
    if (value === undefined || value === null) return false;
    const s = String(value);
    return /^\d+$/.test(s);
  }

  type WeatherLabelKey =
    | 'clear'
    | 'mostly_sunny'
    | 'cloudy'
    | 'overcast'
    | 'fog'
    | 'rain'
    | 'light_rain'
    | 'moderate_rain'
    | 'heavy_rain'
    | 'local_rain'
    | 'thunderstorm';

  const labelKeyByCode: Record<string, WeatherLabelKey> = {
    '0': 'clear',
    '1': 'mostly_sunny',
    '2': 'cloudy',
    '3': 'overcast',
    '45': 'fog',
    '60': 'rain',
    '61': 'light_rain',
    '62': 'moderate_rain',
    '63': 'moderate_rain',
    '65': 'heavy_rain',
    '80': 'local_rain',
    '95': 'thunderstorm',
    '97': 'thunderstorm',
    '99': 'thunderstorm'
  };

  const weatherLabels: Record<'en' | 'id', Record<WeatherLabelKey, { label: string; emoji: string }>> = {
    en: {
      clear: { label: 'Clear', emoji: '☀️' },
      mostly_sunny: { label: 'Mostly Sunny', emoji: '🌤️' },
      cloudy: { label: 'Cloudy', emoji: '☁️' },
      overcast: { label: 'Overcast', emoji: '☁️' },
      fog: { label: 'Fog', emoji: '🌫️' },
      rain: { label: 'Rain', emoji: '🌧️' },
      light_rain: { label: 'Light Rain', emoji: '🌧️' },
      moderate_rain: { label: 'Moderate Rain', emoji: '🌧️' },
      heavy_rain: { label: 'Heavy Rain', emoji: '🌧️' },
      local_rain: { label: 'Local Rain', emoji: '🌦️' },
      thunderstorm: { label: 'Thunderstorm', emoji: '⛈️' }
    },
    id: {
      clear: { label: 'Cerah', emoji: '☀️' },
      mostly_sunny: { label: 'Cerah Berawan', emoji: '🌤️' },
      cloudy: { label: 'Berawan', emoji: '☁️' },
      overcast: { label: 'Mendung', emoji: '☁️' },
      fog: { label: 'Kabut', emoji: '🌫️' },
      rain: { label: 'Hujan', emoji: '🌧️' },
      light_rain: { label: 'Hujan Ringan', emoji: '🌧️' },
      moderate_rain: { label: 'Hujan Sedang', emoji: '🌧️' },
      heavy_rain: { label: 'Hujan Lebat', emoji: '🌧️' },
      local_rain: { label: 'Hujan Lokal', emoji: '🌦️' },
      thunderstorm: { label: 'Badai Petir', emoji: '⛈️' }
    }
  };

  const textLabelKeyMap: Record<'en' | 'id', Record<string, WeatherLabelKey>> = {
    en: {
      'clear': 'clear',
      'mostly sunny': 'mostly_sunny',
      'partly sunny': 'mostly_sunny',
      'cloudy': 'cloudy',
      'overcast': 'overcast',
      'fog': 'fog',
      'rain': 'rain',
      'light rain': 'light_rain',
      'moderate rain': 'moderate_rain',
      'heavy rain': 'heavy_rain',
      'local rain': 'local_rain',
      'thunderstorm': 'thunderstorm'
    },
    id: {
      'cerah': 'clear',
      'cerah berawan': 'mostly_sunny',
      'berawan': 'cloudy',
      'mendung': 'overcast',
      'kabut': 'fog',
      'hujan': 'rain',
      'hujan ringan': 'light_rain',
      'hujan sedang': 'moderate_rain',
      'hujan lebat': 'heavy_rain',
      'hujan lokal': 'local_rain',
      'badai petir': 'thunderstorm'
    }
  };

  function resolveLabelKey(fc: any): WeatherLabelKey | null {
    const rawCode = fc?.weather_code ?? fc?.weather;
    const codeStr = rawCode != null ? String(rawCode) : '';
    if (/^\d+$/.test(codeStr) && labelKeyByCode[codeStr]) return labelKeyByCode[codeStr];
    const textRaw = (fc?.weather ?? '').toString().trim().toLowerCase();
    const l = get(locale);
    const keyFromText = textLabelKeyMap[l][textRaw] || textLabelKeyMap.en[textRaw] || null;
    return keyFromText;
  }

  function getWeatherLabelLocalized(fc: any): { label: string; emoji?: string } {
    const key = resolveLabelKey(fc);
    const l = get(locale);
    if (key) return weatherLabels[l][key];
    const codeStr = fc?.weather_code != null ? String(fc.weather_code) : '';
    if (/^\d+$/.test(codeStr) && labelKeyByCode[codeStr]) {
      const fallbackKey = labelKeyByCode[codeStr];
      return weatherLabels[l][fallbackKey];
    }
    return { label: String(fc?.weather || t('weather')), emoji: undefined };
  }

  function getWeatherIcon(fc: any): { url?: string; emoji?: string } {
    if (fc?.icon_url) return { url: fc.icon_url };
    const key = resolveLabelKey(fc);
    const l = get(locale);
    if (key) return { emoji: weatherLabels[l][key].emoji };
    const codeStr = fc?.weather_code != null ? String(fc.weather_code) : '';
    if (/^\d+$/.test(codeStr) && labelKeyByCode[codeStr]) {
      const fallbackKey = labelKeyByCode[codeStr];
      return { emoji: weatherLabels[l][fallbackKey].emoji };
    }
    return {};
  }

  function formatHeaderLine(fc: any): string {
    const d = parseDateSafe(fc?.local_datetime || fc?.datetime_utc);
    if (!d) return getWeatherLabelLocalized(fc).label;
    const day = d.getDate();
    const l = get(locale);
    const month = monthNames[l][d.getMonth()];
    const year = d.getFullYear();
    const hours = d.getHours();
    const ampm = getAmPm(hours);
    const hour12 = formatHour12(hours);
    const weatherLabel = getWeatherLabelLocalized(fc).label;
    return `${weatherLabel} - ${day} ${month} ${year} - ${hour12} ${ampm}`;
  }

  function formatDateChip(fc: any): string {
    const d = parseDateSafe(fc?.local_datetime || fc?.datetime_utc);
    if (!d) return '';
    const day = d.getDate();
    const l = get(locale);
    const month = monthNames[l][d.getMonth()];
    const year = d.getFullYear();
    return `${day} ${month} ${year}`;
  }

  function formatTimeChip(fc: any): string {
    const d = parseDateSafe(fc?.local_datetime || fc?.datetime_utc);
    if (!d) return '';
    const hours = d.getHours();
    const ampm = getAmPm(hours);
    const hour12 = formatHour12(hours);
    return `${hour12} ${ampm}`;
  }

  // ===== Operational recommendation badges (threshold-based) =====
  type OpsBadge = { icon: string; text: string };
  function computeOpsBadges(fc: any): OpsBadge[] {
    const badges: OpsBadge[] = [];
    const temp = Number(fc?.t);
    const hum = Number(fc?.hu);
    const rain = Number(fc?.tp);
    const wind = Number(fc?.ws);

    // Rain-related recommendations
    if (!Number.isNaN(rain)) {
      if (rain >= 2) badges.push({ icon: '☔', text: t('ops_canopy_queue') });
      if (rain >= 0.5) badges.push({ icon: '🚚', text: t('ops_push_delivery') });
    }

    // Heat & humidity recommendations
    if (!Number.isNaN(temp) && temp >= 30) badges.push({ icon: '🧊', text: t('ops_add_ice') });
    if (!Number.isNaN(hum) && hum >= 85) badges.push({ icon: '❄️', text: t('ops_cooling_display') });
    if (!Number.isNaN(hum) && hum >= 80) badges.push({ icon: '📦', text: t('ops_protect_dry_goods') });

    // Wind recommendations (if available)
    if (!Number.isNaN(wind) && wind >= 8) badges.push({ icon: '🪁', text: t('ops_secure_signage') });

    return badges;
  }

	// init
	$effect(() => {
		loadRegionsFromProfileStore();
	});
</script>

<svelte:head>
    <title>{t('weather')}</title>
</svelte:head>

<section class="space-y-6">
	<div>
    <h1 class="text-2xl font-bold">{t('weather')}</h1>
        <p class="text-sm opacity-70">{t('weather_intro')}</p>
	</div>

	<div class="shadow-card rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-6">
		<div class="grid grid-cols-1 gap-6 md:grid-cols-2">
			<div class="form-group">
                <label class="form-label" for="province">{t('province')}</label>
				<select
					id="province"
					class="form-input"
					bind:value={provinceId}
					disabled={true}
				>
                    <option value="">{t('select_province')}</option>
					{#each provinces as p}
						<option value={p.id}>{p.name}</option>
					{/each}
				</select>
                {#if provincesLoading}
                    <p class="form-help">{t('loading_provinces')}</p>
                {:else if provincesError}
                    <p class="form-error">{provincesError}</p>
                {/if}
			</div>

			<div class="form-group">
                <label class="form-label" for="regency">{t('regency_city')}</label>
				<select
					id="regency"
					class="form-input"
					bind:value={regencyId}
					disabled={true}
				>
                    <option value="">{t('select_regency_city')}</option>
					{#each regencies as r}
						<option value={r.id}>{r.name}</option>
					{/each}
				</select>
                {#if regenciesLoading}
                    <p class="form-help">{t('loading_regencies')}</p>
                {:else if regenciesError}
                    <p class="form-error">{regenciesError}</p>
                {:else if !provinceId}
                    <p class="form-help">{t('select_province_first')}</p>
                {/if}
			</div>

			<div class="form-group">
                <label class="form-label" for="district">{t('district')}</label>
				<select
					id="district"
					class="form-input"
					bind:value={districtId}
					disabled={true}
				>
                    <option value="">{t('select_district')}</option>
					{#each districts as d}
						<option value={d.id}>{d.name}</option>
					{/each}
				</select>
                {#if districtsLoading}
                    <p class="form-help">{t('loading_districts')}</p>
                {:else if districtsError}
                    <p class="form-error">{districtsError}</p>
                {:else if !regencyId}
                    <p class="form-help">{t('select_regency_first')}</p>
                {/if}
			</div>

			<div class="form-group">
                <label class="form-label" for="village">{t('village_subdistrict')}</label>
				<select
					id="village"
					class="form-input"
					bind:value={villageId}
					disabled={true}
				>
                    <option value="">{t('select_village_subdistrict')}</option>
					{#each villages as v}
						<option value={v.id}>{v.name}</option>
					{/each}
				</select>
                {#if villagesLoading}
                    <p class="form-help">{t('loading_villages')}</p>
                {:else if villagesError}
                    <p class="form-error">{villagesError}</p>
                {:else if !districtId}
                    <p class="form-help">{t('select_district_first')}</p>
                {/if}
			</div>
		</div>

		<div class="mt-6 flex items-center gap-3">
            <Button
                label={t('view_prediction')}
                color="orange"
                onClick={showWeather}
                disabled={!provinceId || !regencyId || !districtId || !villageId}
            />
      <span class="text-sm opacity-70">{t('prediction_results_below')}</span>
		</div>
	</div>

	<div class="shadow-card rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-6">
    <h2 class="mb-4 text-lg font-semibold">{t('results')}</h2>
		{#if weatherLoading}
      <p class="text-sm">{t('loading_prediction')}</p>
		{:else if weatherError}
			<p class="text-sm text-red-600">{weatherError}</p>
		{:else if weather}
			<div class="space-y-4">
                <div class="text-sm opacity-80">
                    <p>
            {t('region')}:
                        <strong>
                            {weather?.lokasi?.village_name || villages.find((v) => v.id === villageId)?.name || '-'}
                            , {weather?.lokasi?.district_name || districts.find((d) => d.id === districtId)?.name || '-'}
                            , {weather?.lokasi?.regency_name || regencies.find((r) => r.id === regencyId)?.name || '-'}
                            , {weather?.lokasi?.province_name || provinces.find((p) => p.id === provinceId)?.name || '-'}
                        </strong>
                    </p>
                    {#if weather?.last_updated}
            <p>{t('last_updated')}: {new Date(weather.last_updated).toLocaleString($locale === 'id' ? 'id-ID' : 'en-US')}</p>
                    {/if}
                </div>

                {#if weather?.prakiraan_cuaca?.length}
                  <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
                    {#each weather.prakiraan_cuaca as fc, i}
                      <div class="min-w-0 rounded-xl border border-[var(--ui-border)] bg-[color-mix(in_oklch,var(--color-surface),white_4%)] p-4 transition hover:bg-[color-mix(in_oklch,var(--color-surface),white_8%)]">
                        <div class="flex items-center justify-between gap-3">
                          <div class="flex items-center gap-3">
                            {#if getWeatherIcon(fc).url}
                              <img src={getWeatherIcon(fc).url} alt={getWeatherLabelLocalized(fc).label} class="h-10 w-10" />
                            {:else if getWeatherIcon(fc).emoji}
                              <div class="text-2xl" aria-hidden="true">{getWeatherIcon(fc).emoji}</div>
                            {/if}
                            <div>
                              <p class="text-base md:text-lg font-semibold leading-snug">{getWeatherLabelLocalized(fc).label}</p>
                              <div class="mt-1 flex flex-wrap gap-1">
                                <span class="inline-flex items-center gap-1 rounded-full border border-[color-mix(in_oklch,var(--ui-border),black_10%)] bg-[var(--color-surface)] px-2 py-0.5 text-xs">
                                  📅 {formatDateChip(fc)}
                                </span>
                                <span class="inline-flex items-center gap-1 rounded-full border border-[color-mix(in_oklch,var(--ui-border),black_10%)] bg-[var(--color-surface)] px-2 py-0.5 text-xs">
                                  ⏰ {formatTimeChip(fc)}
                                </span>
                              </div>
                            </div>
                          </div>
                        </div>

                        <!-- Operational recommendation badges (threshold-based) -->
                        {#if computeOpsBadges(fc).length}
                          <div class="mt-2 flex flex-wrap gap-1 text-sm">
                            {#each computeOpsBadges(fc) as rec}
                              <span class="inline-flex items-center gap-1 rounded-full border border-[var(--ui-border)] bg-[color-mix(in_oklch,var(--color-surface),white_6%)] px-2.5 py-1 text-sm">
                                <span aria-hidden="true">{rec.icon}</span>
                                <span class="leading-tight">{rec.text}</span>
                              </span>
                            {/each}
                          </div>
                        {/if}

                        <!-- Essentials only for F&B -->
                        <div class="mt-3 grid grid-cols-2 gap-2 text-xs md:text-sm">
                          <div class="inline-flex items-center gap-1 rounded-full border border-[color-mix(in_oklch,var(--ui-border),black_10%)] bg-[var(--color-surface)] px-2 py-1">
                            <span aria-hidden="true">🌡️</span>
                            <span class="opacity-80">{t('temperature')}:</span>
                            <strong class="font-semibold inline-block truncate max-w-[10ch] leading-tight">{fc.t ?? '-' }°C</strong>
                          </div>
                          <div class="inline-flex items-center gap-1 rounded-full border border-[color-mix(in_oklch,var(--ui-border),black_10%)] bg-[var(--color-surface)] px-2 py-1">
                            <span aria-hidden="true">💧</span>
                            <span class="opacity-80">{t('humidity')}:</span>
                            <strong class="font-semibold inline-block truncate max-w-[10ch] leading-tight">{fc.hu ?? '-' }%</strong>
                          </div>
                          <div class="inline-flex items-center gap-1 rounded-full border border-[color-mix(in_oklch,var(--ui-border),black_10%)] bg-[var(--color-surface)] px-2 py-1">
                            <span aria-hidden="true">🌧️</span>
                            <span class="opacity-80">{t('rainfall')}:</span>
                            <strong class="font-semibold inline-block truncate max-w-[10ch] leading-tight">{fc.tp ?? '-' } mm</strong>
                          </div>
                          <div class="inline-flex items-center gap-1 rounded-full border border-[color-mix(in_oklch,var(--ui-border),black_10%)] bg-[var(--color-surface)] px-2 py-1">
                            <span aria-hidden="true">🧭</span>
                            <span class="opacity-80">{t('wind_direction')}:</span>
                            <strong class="font-semibold inline-block truncate max-w-[10ch] leading-tight">{fc.wd ?? '-' }</strong>
                          </div>
                          <div class="inline-flex items-center gap-1 rounded-full border border-[color-mix(in_oklch,var(--ui-border),black_10%)] bg-[var(--color-surface)] px-2 py-1">
                            <span aria-hidden="true">💨</span>
                            <span class="opacity-80">{t('wind_speed')}:</span>
                            <strong class="font-semibold inline-block truncate max-w-[10ch] leading-tight">{fc.ws ?? '-' } m/s</strong>
                          </div>
                          <div class="inline-flex items-center gap-1 rounded-full border border-[color-mix(in_oklch,var(--ui-border),black_10%)] bg-[var(--color-surface)] px-2 py-1">
                            <span aria-hidden="true">☁️</span>
                            <span class="opacity-80">{t('total_cloud_cover')}:</span>
                            <strong class="font-semibold inline-block truncate max-w-[10ch] leading-tight">{fc.tcc ?? '-' }%</strong>
                          </div>
                        </div>

                        <div class="mt-4">
                          <details class="group">
                            <summary class="flex cursor-pointer list-none items-center gap-2">
                              <span class="text-sm font-medium opacity-80">{t('essential_metrics')}</span>
                              <span class="inline-flex items-center gap-1 rounded-full border border-[var(--ui-border)] bg-[var(--color-surface)] px-2 py-0.5 text-xs">
                                {#if fc.tp != null && looksNumeric(fc.tp)}
                                  <span class="opacity-80">{t('rainfall')}:</span>
                                  <strong class="inline-block truncate max-w-[10ch] leading-tight">{fc.tp} mm</strong>
                                {/if}
                                {#if fc.t != null && looksNumeric(fc.t)}
                                  <span class="opacity-80">• {t('temperature')}:</span>
                                  <strong class="inline-block truncate max-w-[10ch] leading-tight">{fc.t}°C</strong>
                                {/if}
                                {#if fc.hu != null && looksNumeric(fc.hu)}
                                  <span class="opacity-80">• {t('humidity')}:</span>
                                  <strong class="inline-block truncate max-w-[10ch] leading-tight">{fc.hu}%</strong>
                                {/if}
                              </span>
                            </summary>
                          </details>
                        </div>
                      </div>
                    {/each}
                  </div>
                {:else}
                  <div class="rounded-xl border border-[var(--ui-border)] bg-[var(--color-surface)] p-4 text-sm opacity-80">
                    {t('no_weather_forecast')}
                  </div>
                {/if}
			</div>
		{:else}
			<div class="rounded-xl border border-[var(--ui-border)] bg-[var(--color-surface)] p-4 text-sm opacity-80">
        {t('no_results_yet')}
			</div>
		{/if}
	</div>
</section>
