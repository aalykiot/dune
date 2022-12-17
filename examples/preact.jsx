import { h, Component } from 'https://esm.sh/preact@10.11.3';
import { render } from 'https://esm.sh/preact-render-to-string@5.2.6';

/** @jsx h */

// Classical components work.
class Fox extends Component {
  render({ name }) {
    return <span class="fox">{name}</span>;
  }
}

// ... and so do pure functional components:
const Box = ({ type, children }) => (
  <div class={`box box-${type}`}>{children}</div>
);

let html = render(
  <Box type="open">
    <Fox name="Finn" />
  </Box>
);

console.log(html);
