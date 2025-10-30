<script lang="ts">
  import { browser } from '$app/environment';
  import { locale, setLocale, supportedLocales, t } from '$lib/stores/i18n';

  function choose(l: 'en' | 'id') {
    setLocale(l);
  }
</script>

<div class="relative">
  <div
    class="shadow-header inline-flex items-center gap-1 rounded-full border border-[var(--ui-border)] bg-[var(--color-surface)] px-1.5 py-1.5"
    role="group"
    aria-label={t('language')}
  >
    <button
      type="button"
      onclick={() => choose('en')}
      class="rounded-full px-2 py-0.5 text-sm transition hover:bg-[color-mix(in_oklch,var(--color-surface),black_4%)] focus-visible:ring-2 focus-visible:ring-[--nav-active-ring] focus-visible:outline-none data-[active=true]:nav-active-bg data-[active=true]:text-[--nav-active-text] data-[active=true]:ring-1 data-[active=true]:ring-[--nav-active-ring]"
      data-active={$locale === 'en'}
      aria-pressed={$locale === 'en'}
    >
      EN
    </button>
    <button
      type="button"
      onclick={() => choose('id')}
      class="rounded-full px-2 py-0.5 text-sm transition hover:bg-[color-mix(in_oklch,var(--color-surface),black_4%)] focus-visible:ring-2 focus-visible:ring-[--nav-active-ring] focus-visible:outline-none data-[active=true]:nav-active-bg data-[active=true]:text-[--nav-active-text] data-[active=true]:ring-1 data-[active=true]:ring-[--nav-active-ring]"
      data-active={$locale === 'id'}
      aria-pressed={$locale === 'id'}
    >
      ID
    </button>
  </div>
</div>