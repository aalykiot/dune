import test from 'test';
import assert from 'assert';
import { render } from 'https://esm.sh/preact-render-to-string@5.2.6';
import App from './helpers/App.jsx';

test('[JSX] JSX syntax should be supported.', () => {
  const html = '<div class="box box-open"><span class="fox">Finn</span></div>';
  assert.equal(render(App()), html);
});
