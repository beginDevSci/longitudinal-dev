<template>
    <div>
      <AppHeader />
      <UMain>
        <slot /> <!-- ✅ Renders content dynamically -->
      </UMain>
      <AppFooter />
    </div>
  </template>
  