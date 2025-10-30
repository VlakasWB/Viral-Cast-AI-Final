<script lang="ts">
  // [ID] Ambil data dari server load untuk daftar kategori
  // [EN] Read data provided by server load for category listing
  import Button from '$lib/components/ui/Button.svelte';
  import { goto } from '$app/navigation';
  import type { Category } from '$lib/types/category.js';

  let {
    data
  }: {
    data: {
      items: Category[];
      total: number;
      page: number;
      pageCount: number;
      q: string;
      size: number;
    };
  } = $props();

  // [ID] Hapus kategori: kirim form action "delete" ke +page.server.ts
  // [EN] Delete category: post "delete" action to +page.server.ts
  function submitDelete(uuid: string) {
    // [ID] Konfirmasi sederhana untuk mencegah salah klik
    // [EN] Simple confirmation to avoid accidental deletion
    if (!confirm('Hapus kategori ini?')) return;

    const form = document.createElement('form');
    form.method = 'post';
    form.action = '?/delete';

    const input = document.createElement('input');
    input.type = 'hidden';
    input.name = 'uuid';
    input.value = uuid;
    form.appendChild(input);

    document.body.appendChild(form);
    form.submit();
  }
</script>

<svelte:head>
  <title>Master — Categories</title>
</svelte:head>

<section class="space-y-6">
  <div class="flex items-center justify-between">
    <h1 class="text-2xl font-semibold">Categories</h1>

    <div class="flex items-center gap-3">
      <!-- [ID] Tombol tambah kategori baru -->
      <!-- [EN] Button to create a new category -->
      <Button label="New" color="accent" onClick={() => goto('/master/categories/new')} />
    </div>
  </div>

  <!-- [ID] Ringkasan pagination server-driven (sederhana) -->
  <!-- [EN] Simple server-driven pagination summary -->
  <div class="text-sm opacity-70">
    Total: {data.total} • Page {data.page} / {data.pageCount}
  </div>

  {#if (data.items?.length ?? 0) === 0}
    <!-- [ID] Kosong: tampilkan pesan ramah -->
    <!-- [EN] Empty state: show friendly message -->
    <div class="rounded-xl border border-[var(--ui-border)] p-6 text-center opacity-80">
      Tidak ada kategori.
    </div>
  {:else}
    <!-- [ID] Tabel daftar kategori -->
    <!-- [EN] Categories table listing -->
    <div class="overflow-auto rounded-xl border border-[var(--ui-border)]">
      <table class="min-w-full text-sm">
        <thead class="bg-[color-mix(in_oklch,var(--color-surface),black_3%)]">
          <tr>
            <th class="px-4 py-3 text-left">Name</th>
            <th class="px-4 py-3 text-left">Updated</th>
            <th class="px-4 py-3 text-left">Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each data.items as cat}
            <tr class="border-t border-[var(--ui-border)]">
              <td class="px-4 py-3">{cat.name}</td>
              <td class="px-4 py-3 text-xs opacity-70">{new Date(cat.updated_at).toLocaleString()}</td>
              <td class="px-4 py-3">
                <div class="flex items-center gap-2">
                  <!-- [ID] Tautan edit -->
                  <!-- [EN] Edit link -->
                  <a
                    href={`/master/categories/${cat.uuid}/edit`}
                    class="rounded-lg border border-[var(--ui-border)] px-3 py-1.5 hover:bg-black/5 dark:hover:bg-white/5"
                    >Edit</a
                  >

                  <!-- [ID] Aksi hapus (POST ke action delete) -->
                  <!-- [EN] Delete action (POST to delete action) -->
                  <button
                    class="rounded-lg border border-[var(--ui-border)] px-3 py-1.5 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20"
                    on:click={() => submitDelete(cat.uuid)}
                  >Delete</button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</section>
