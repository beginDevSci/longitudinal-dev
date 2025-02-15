<template>
  <div>
    <!-- Tab navigation -->
    <button @click="setActiveTab('tab1')">Tab 1</button>
    <button @click="setActiveTab('tab2')">Tab 2</button>
    <button @click="setActiveTab('tab3')">Tab 3</button>

    <!-- Tab contents, conditionally displayed based on the active tab -->
    <div v-if="activeTab === 'tab1'">
      <slot name="tab1-content"></slot>
    </div>
    
    <div v-if="activeTab === 'tab2'">
      <slot name="tab2-content"></slot>
    </div>

    <div v-if="activeTab === 'tab3'">
      <slot name="tab3-content"></slot>
    </div>
  </div>
</template>

<script setup>
import { CardComponent1 } from '#build/components';
import { ref } from 'vue'

// Create a reactive variable to track the active tab
const activeTab = ref('tab1')

// Function to change the active tab
const setActiveTab = (tab) => {
  activeTab.value = tab
}
</script>

<style scoped>
button {
  padding: 10px 20px;
  margin-right: 10px;
  background-color: #00dc82;
  color: white;
  border: none;
  cursor: pointer;
}
button:hover {
  background-color: #00a155;
}
div {
  margin-top: 20px;
  font-size: 18px;
  color: #333;
}
</style>
