import fs from 'fs/promises';
import { promisify } from 'util';
import child_process from 'child_process';
import { select, input } from '@inquirer/prompts';
import { Spinner } from '@topcli/spinner';
import semver from 'semver';
import packageJson from '../package.json';

const exec = promisify(child_process.exec);

const possibleVersions = {
  major: semver.inc(packageJson.version, 'major'),
  minor: semver.inc(packageJson.version, 'minor'),
  patch: semver.inc(packageJson.version, 'patch'),
};

const versionOptions = {
  message: 'What kind of version update would you like?',
  choices: [
    {
      name: 'major',
      value: 'major',
      description: 'When making incompatible API changes',
    },
    {
      name: 'minor',
      value: 'minor',
      description: 'When adding functionality in a backward compatible manner',
    },
    {
      name: 'patch',
      value: 'patch',
      description: 'When making backward compatible bug fixes',
    },
  ],
};

const version = await select(versionOptions);
const tagMessage = await input({ message: 'Enter a git version tag message:' });

const spinner = new Spinner().start('Setting new version to files');

packageJson.version = possibleVersions[version];

// Update package.json and Cargo.toml
await exec(`cargo set-version ${packageJson.version}`);
await fs.writeFile('./package.json', JSON.stringify(packageJson, null, 2));

spinner.text = 'Updating cargo lock file';

// Run cargo update to sync the lock file as well.
await exec('cargo update');

spinner.text = 'Committing changes and creating new version tag';

// Commit chnages and create a new git tag.
await exec('git add .');
await exec(`git commit -m "Bumping version to v${packageJson.version}"`);
await exec(`git tag -a v${packageJson.version} -m "${tagMessage}"`);

spinner.succeed(`Version bumped to v${packageJson.version}! ✨`);

console.log('Run "git push origin main --tags" to push the new tag.\n');
