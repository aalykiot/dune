import http from 'http';

const URL = 'https://jsonplaceholder.typicode.com/posts';

const response = await http.request(URL, {
  timeout: 0, // <-- No timeout.
  throwOnError: true, // <-- Throw on receiving 4xx or 5xx.
});

const todos = await response.body.json();
const titles = todos.map((todo) => todo.title);

console.log(titles);
