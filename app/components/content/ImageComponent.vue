<!-- components/content/ImageComponent.vue -->
<template>
    <NuxtImg :src="src" :alt="alt" :width="width" class="mx-auto" />
  </template>
  
  <script setup>
  defineProps({
    src: String,
    alt: String,
    width: { type: Number, default: 200 }
  });
  </script>
  