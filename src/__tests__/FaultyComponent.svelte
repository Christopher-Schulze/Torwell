<script lang="ts">
  export let trigger: boolean = false;
  if (trigger) {
    throw new Error('Boom');
  }
</script>
<div>ok</div>
