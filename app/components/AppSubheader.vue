<template>
  <div class="subheader bg-gray-800 text-white py-2 relative">
    <!-- Left Icon Button -->
    <button
      class="absolute left-0 top-1/2 transform -translate-y-1/2 bg-gray-700 p-2 rounded-l z-10"
      @click="scrollLeft"
    >
      <i class="fas fa-chevron-left text-white" />
    </button>

    <!-- Subheader Content -->
    <div
      ref="scrollContainer"
      class="flex flex-col md:flex-row justify-between items-center px-4 overflow-x-auto ml-8 mr-8"
    >
      <ul class="flex flex-nowrap space-x-4">
        <li>
          <NuxtLink
            to="/tutorials/longitudinal-analyses-models/growth-curve-models/latent_growth_curve_models_tutorial"
            class="text-white hover:text-gray-300 whitespace-nowrap"
          >
            Growth Curves
          </NuxtLink>
        </li>
        <li>
          <NuxtLink
            to="/resources"
            class="text-white hover:text-gray-300 whitespace-nowrap"
          >
            Cheat Sheets
          </NuxtLink>
        </li>
        <li>
          <NuxtLink
            to="/tutorials/longitudinal-analyses-models/mixed-effects-and-multilevel/linear-mixedeffects-models"
            class="text-white hover:text-gray-300 whitespace-nowrap"
          >
            Linear Mixed Models
          </NuxtLink>
        </li>
        <li>
          <NuxtLink
            to="/tools#ides"
            class="text-white hover:text-gray-300 whitespace-nowrap"
          >
            IDEs
          </NuxtLink>
        </li>
        <AppDropdown>
          <template #link>
            <li>
              <NuxtLink
                to="/tutorials/data-wrangling/cleaning-and-transforming-data"
                class="text-white hover:text-gray-300 whitespace-nowrap"
              >
                Data Wrangling
              </NuxtLink>
            </li>
          </template>
          <template #dropdown>
            <ul class="py-1">
              <li>
                <a
                  href="#"
                  class="block px-4 py-2 text-gray-700 hover:bg-gray-100"
                  >Overview</a
                >
              </li>
              <li>
                <a
                  href="#"
                  class="block px-4 py-2 text-gray-700 hover:bg-gray-100"
                  >Python</a
                >
              </li>
              <li>
                <a
                  href="#"
                  class="block px-4 py-2 text-gray-700 hover:bg-gray-100"
                  >Machine Learning</a
                >
              </li>
            </ul>
          </template>
        </AppDropdown>
        <AppDropdown>
          <template #link>
            <li>
              <NuxtLink
                to="/resources"
                class="text-white hover:text-gray-300 whitespace-nowrap"
              >
                Programming Languages
              </NuxtLink>
            </li>
          </template>
          <template #dropdown>
            <ul class="py-1">
              <li>
                <a
                  href="#"
                  class="block px-4 py-2 text-gray-700 hover:bg-gray-100"
                  >Python</a
                >
              </li>
              <li>
                <a
                  href="#"
                  class="block px-4 py-2 text-gray-700 hover:bg-gray-100"
                  >JavaScript</a
                >
              </li>
              <li>
                <a
                  href="#"
                  class="block px-4 py-2 text-gray-700 hover:bg-gray-100"
                  >Java</a
                >
              </li>
              <li>
                <a
                  href="#"
                  class="block px-4 py-2 text-gray-700 hover:bg-gray-100"
                  >C++</a
                >
              </li>
            </ul>
          </template>
        </AppDropdown>
        <li>
          <NuxtLink
            to="/resources"
            class="text-white hover:text-gray-300 whitespace-nowrap"
          >
            Workshops
          </NuxtLink>
        </li>
        <li>
          <NuxtLink
            to="/abcd-study"
            class="text-white hover:text-gray-300 whitespace-nowrap"
          >
            ABCD Study®
          </NuxtLink>
        </li>
        <li>
          <NuxtLink
            to="/tutorials"
            class="text-white hover:text-gray-300 whitespace-nowrap"
          >
            Tutorials
          </NuxtLink>
        </li>
        <li>
          <NuxtLink
            to="/tutorials/data-visualization/viz_customizing-plots-ggplot2"
            class="text-white hover:text-gray-300 whitespace-nowrap"
          >
            Visualizations
          </NuxtLink>
        </li>
        <li>
          <NuxtLink
            to="/tools#notebooks"
            class="text-white hover:text-gray-300 whitespace-nowrap"
          >
            Quarto
          </NuxtLink>
        </li>
        <li>
          <NuxtLink
            to="/tools#data-formats"
            class="text-white hover:text-gray-300 whitespace-nowrap"
          >
            Markdown
          </NuxtLink>
        </li>
        <li>
          <a href="#" class="text-white hover:text-gray-300 whitespace-nowrap"
            >Book Club</a
          >
        </li>
        <li>
          <a href="#" class="text-white hover:text-gray-300 whitespace-nowrap"
            >Missing Data</a
          >
        </li>
        <li>
          <a
            href="https://longitudinaldev.slack.com"
            class="text-white hover:text-gray-300 whitespace-nowrap"
            target="_blank"
            >Slack Channel</a
          >
        </li>
        <li>
          <a
            href="https://github.com/beginDevSci/longitudinal-dev"
            class="text-white hover:text-gray-300 whitespace-nowrap"
            target="_blank"
            >Github</a
          >
        </li>
        <!-- <li>
          <NuxtLink
            to="/playground"
            class="text-white hover:text-gray-300 whitespace-nowrap"
          >
            Playground
          </NuxtLink>
        </li> -->
      </ul>
    </div>

    <!-- Right Icon Button -->
    <button
      class="absolute right-0 top-1/2 transform -translate-y-1/2 bg-gray-700 p-2 rounded-r z-10"
      @click="scrollRight"
    >
      <i class="fas fa-chevron-right text-white" />
    </button>
  </div>
</template>

<script setup>
import { ref } from "vue";
import AppDropdown from "./AppDropdown.vue";

const scrollContainer = ref(null);

const scrollLeft = () => {
  scrollContainer.value.scrollBy({ left: -200, behavior: "smooth" });
};

const scrollRight = () => {
  scrollContainer.value.scrollBy({ left: 200, behavior: "smooth" });
};
</script>

<style scoped>
.subheader {
  position: relative;
}
</style>
