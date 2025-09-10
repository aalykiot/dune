try {
  const { add } = await import('./utils/adder.js');
  const { echo } = await import('./utils/echo.js');
  echo(add(2, 3));
} catch (e) {
  console.log(`Failed to dynamic import: ${e}`);
}
