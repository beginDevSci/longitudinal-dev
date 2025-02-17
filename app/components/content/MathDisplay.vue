<template>
  <div class="math-container">
    <span v-html="formattedEquation" />
  </div>
</template>

<script setup>
import { computed } from "vue";
import katex from "katex";
import "katex/dist/katex.min.css";

const props = defineProps({
  equation: String, // LaTeX equation as a string
  displayMode: { type: Boolean, default: true }, // Inline vs block display
});

// Format equation using KaTeX for rendering
const formattedEquation = computed(() => {
  try {
    return katex.renderToString(props.equation, {
      throwOnError: false,
      displayMode: props.displayMode,
    });
  } catch (error) {
    return `Error rendering equation: ${error.message}`;
  }
});
</script>

<style scoped>
.math-container {
  font-size: 1.2rem;
  text-align: center;
  margin: 10px 0;
}
</style>
