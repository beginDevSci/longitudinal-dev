<template>
  <ULandingCard
    :title="tutorial.title"
    icon="i-heroicons-academic-cap"
    color="primary"
    class="h-auto min-h-48 flex flex-col justify-between"
  >
    <div class="flex flex-col flex-grow">
      <!-- ✅ Description -->
      <p class="text-xs text-gray-700 text-justify flex-grow">
        {{ tutorial.description }}
      </p>
    </div>

    <!-- ✅ Button to Access Tutorial -->
    <div class="mt-4">
      <a
        :href="tutorial.link"
        target="_blank"
        rel="noopener noreferrer"
        class="inline-block bg-blue-600 hover:bg-blue-700 text-white text-sm font-semibold px-4 py-2 rounded-lg text-center"
      >
        Open Tutorial 🚀
      </a>
    </div>
  </ULandingCard>
</template>

<script setup>
defineProps({
  tutorial: Object, // Accepts a tutorial object as a prop
});
</script>
