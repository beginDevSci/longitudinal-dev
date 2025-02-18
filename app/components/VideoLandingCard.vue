<template>
  <ULandingCard
    :title="video.title"
    icon="i-heroicons-video-camera"
    color="primary"
  >
    <div class="w-full">
      <!-- ✅ Video Embed -->
      <div class="w-full h-40 flex items-center justify-center overflow-hidden">
        <iframe
          :src="video.link"
          frameborder="0"
          allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
          allowfullscreen
          class="w-full h-full"
        />
      </div>

      <!-- ✅ Description -->
      <p class="text-xs text-white-700 text-left mt-2">
        {{ video.description }}
      </p>
    </div>
  </ULandingCard>
</template>

<script setup>
defineProps({
  video: Object,
});
</script>
