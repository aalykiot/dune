import fs from 'fs';

const DATA = 'Welcome to Dune 🪐';

await fs.writeFile('newfile.txt', DATA, 'utf-8');
