<script lang="ts">
  import { createProfileApi, updateProfileApi } from '$lib/services/profile.js';

  let { data } = $props();

  let form = $state({
    name: data?.profile?.name ?? data?.user?.name ?? '',
    gender: (data?.profile?.gender ?? 'MALE') as 'MALE' | 'FEMALE' | 'OTHER',
    telp: data?.profile?.telp ?? '',
    birth_date: data?.profile?.birth_date ?? ''
  });

  let saving = $state(false);
  let saveError = $state<string | null>(null);
  let saveSuccess = $state<string | null>(null);


  async function handleSave() {
    saving = true;
    saveError = null;
    saveSuccess = null;
    try {
      const payload = {
        name: form.name,
        gender: form.gender,
        telp: form.telp,
        birth_date: form.birth_date
      };

      let res;
      if (data?.profile) {
        res = await updateProfileApi(payload);
      } else {
        res = await createProfileApi(payload);
      }

      const updated = (res?.data?.profile) ?? res?.data ?? null;
      if (updated) {
        saveSuccess = 'Profil berhasil disimpan';
        data.profile = updated;
      } else {
        saveSuccess = 'Profil tersimpan';
      }
    } catch (err: any) {
      saveError = err?.message || 'Gagal menyimpan profil';
    } finally {
      saving = false;
    }
  }
</script>

<section class="shadow-card rounded-[var(--radius-card)] border border-[var(--ui-border)] bg-[var(--color-surface)] p-6">
  <h3 class="text-base font-semibold">Profile Settings</h3>
  <p class="mt-1 text-sm opacity-70">Lengkapi informasi profil Anda.</p>

  <div class="mt-4 grid grid-cols-1 md:grid-cols-2 gap-4">
    <div>
      <label for="name" class="block text-sm mb-1">Nama</label>
      <input id="name" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-transparent outline-none" type="text" bind:value={form.name} />
    </div>
    <div>
      <label for="gender" class="block text-sm mb-1">Gender</label>
      <select id="gender" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-transparent outline-none" bind:value={form.gender}>
        <option value="MALE">MALE</option>
        <option value="FEMALE">FEMALE</option>
        <option value="OTHER">OTHER</option>
      </select>
    </div>
    <div>
      <label for="telp" class="block text-sm mb-1">Telepon</label>
      <input id="telp" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-transparent outline-none" type="text" bind:value={form.telp} />
    </div>
    <div>
      <label for="birth_date" class="block text-sm mb-1">Tanggal Lahir</label>
      <input id="birth_date" class="w-full rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-4 py-2.5 bg-transparent outline-none" type="date" bind:value={form.birth_date} />
    </div>
    <!-- roles_number, store info, dan wilayah dihilangkan sesuai permintaan -->
  </div>

  {#if saveError}
    <div class="mt-4 text-red-600">{saveError}</div>
  {/if}
  {#if saveSuccess}
    <div class="mt-4 text-green-600">{saveSuccess}</div>
  {/if}

  <div class="mt-6 flex gap-3">
    <button type="button" class="rounded-[var(--radius-pill)] border border-[var(--ui-border)] px-5 py-2.5 hover:bg-[color-mix(in_oklch,var(--color-surface),black_3%)]" onclick={handleSave} disabled={saving}>
      {saving ? 'Menyimpan...' : 'Simpan Profil'}
    </button>
  </div>
</section>