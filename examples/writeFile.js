import fs from 'fs';

const DATA = 'Welcome to Dune 🪐';

await fs.writeFile('newFile.txt', DATA, 'utf-8');
