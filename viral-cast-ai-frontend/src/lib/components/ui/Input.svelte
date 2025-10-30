<script lang="ts">
  // ID: Komponen Input untuk form
  // EN: Input component for forms
  export let id: string = '';
  export let name: string = '';
  export let label: string = '';
  export let value: string = '';
  export let type: string = 'text';
  export let placeholder: string = '';
  export let required: boolean = false;
  export let disabled: boolean = false;
  export let error: string = '';
  export let min: string | number = '';
  export let max: string | number = '';
  export let step: string | number = '';
  export let autocomplete: string = 'off';
  export let classes: string = '';
</script>

<div class="form-group mb-3">
  {#if label}
    <label for={id} class="form-label">
      {label}
      {#if required}
        <span class="text-danger">*</span>
      {/if}
    </label>
  {/if}
  
  <input
    {id}
    {name}
    {type}
    {placeholder}
    {required}
    {disabled}
    {autocomplete}
    bind:value
    min={min.toString()}
    max={max.toString()}
    step={step.toString()}
    class={`form-input modern-input ${error ? 'is-invalid' : ''} ${classes}`}
    on:input
    on:change
    on:focus
    on:blur
  />
  
  {#if error}
    <div class="invalid-feedback">
      {error}
    </div>
  {/if}
</div>

<style>
	.modern-input {
		border-radius: 1rem;
		border: 1px solid var(--input-border, rgba(0, 0, 0, 0.15));
		background: var(--input-bg, #ffffff);
		color: var(--color-text, var(--text));
		padding: 0.55rem 1rem;
		transition:
			border-color var(--transition-fast, 150ms ease-out),
			box-shadow var(--transition-fast, 150ms ease-out),
			background var(--transition-fast, 150ms ease-out);
	}

	.modern-input:focus {
		border-color: var(--input-focus-border, var(--accent));
		box-shadow: 0 0 0 3px var(--input-focus-ring, rgba(255, 62, 0, 0.2));
		background: color-mix(in srgb, var(--input-bg, #ffffff) 80%, #ffffff);
		outline: none;
	}

	html[data-mode='dark'] .modern-input {
		background: rgba(15, 23, 42, 0.6);
		border-color: rgba(148, 163, 184, 0.35);
		color: oklch(0.92 0 0);
	}

	html[data-mode='dark'] .modern-input:focus {
		border-color: var(--accent);
		box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 35%, transparent);
	}
</style>
