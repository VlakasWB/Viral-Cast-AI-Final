<script lang="ts">
  // ID: Komponen TextArea untuk input teks panjang
  // EN: TextArea component for long text input
  export let id: string = '';
  export let name: string = '';
  export let label: string = '';
  export let value: string = '';
  export let placeholder: string = '';
  export let required: boolean = false;
  export let disabled: boolean = false;
  export let error: string = '';
  export let rows: number = 3;
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
  
  <textarea
    {id}
    {name}
    {placeholder}
    {required}
    {disabled}
    {rows}
    bind:value
    class="form-control {error ? 'is-invalid' : ''} {classes}"
    on:input
    on:change
    on:focus
    on:blur
  ></textarea>
  
  {#if error}
    <div class="invalid-feedback">
      {error}
    </div>
  {/if}
</div>