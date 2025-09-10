import('./util')
  .then((util) => {
    util.echo(util.add(1, 2));
  })
  .catch((e) => {
    console.log(`Failed to dynamic import './util': ${e}`);
  });
