<template>
    <div>
      <button :class="{ active: activeTab === 'tab1' }" @click="emitTab('tab1')">Tab 1</button>
      <button :class="{ active: activeTab === 'tab2' }" @click="emitTab('tab2')">Tab 2</button>
    </div>
  </template>
  
  <script setup>
  const emit = defineEmits(['update:activeTab']);
  const emitTab = (tab) => {
    emit('update:activeTab', tab);
  };
  </script>
  
  <style scoped>
  button {
    padding: 10px;
    cursor: pointer;
  }
  
  button.active {
    background-color: #00dc82;
    color: white;
  }
  </style>
  