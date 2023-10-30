import { h, Component } from 'https://esm.sh/preact@10.11.3';

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

const App = () => (
  <Box type="open">
    <Fox name="Finn" />
  </Box>
);

export default App;
