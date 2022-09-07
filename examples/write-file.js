import fs from 'fs';

const DATA = 'Welcome to Dune 🪐';

async function main() {
  try {
    await fs.writeFile('newfile.txt', DATA, 'utf-8');
  } catch (e) {
    console.log('Err', e);
  }
}

main();
