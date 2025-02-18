<template>
  <ULandingCard
    icon="i-heroicons-book-open"
    color="primary"
  >
    <a
      :href="book.link"
      target="_blank"
      rel="noopener noreferrer"
      class="w-full"
    >

      <!-- ✅ Full-Width Image -->
      <div class="w-full h-64 flex items-center justify-center overflow-hidden">
        <img
          :src="book.image"
          :alt="book.title"
          class="w-full h-full object-cover"
        />
      </div>

      <!-- ✅ Description -->
      <p class="text-xs text-white-700 text-left mt-2">
        {{ book.description }}
      </p>
    </a>
  </ULandingCard>
</template>

<script setup>
defineProps({
  book: Object,
});
</script>
