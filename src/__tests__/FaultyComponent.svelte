<script lang="ts">
  export let trigger = false;
  if (trigger) {
    throw new Error('Boom');
  }
</script>
<div>ok</div>
