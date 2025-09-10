try {
  const { echoA } = await import('./utils/a.js');
  echoA(3);
} catch (e) {
  console.log(`Failed to dynamic import:${e}`);
}
