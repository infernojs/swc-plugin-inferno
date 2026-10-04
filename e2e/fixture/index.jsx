import { render } from "inferno";

function Item({ label }) {
  return <li className="item">{label}</li>;
}

function Dynamic({ childFlag, children }) {
  return <p $ChildFlag={childFlag}>{children}</p>;
}

function App({ items }) {
  return (
    <div className="app" id="root">
      <h1>Hello</h1>
      <ul $HasKeyedChildren>
        {items.map((item) => (
          <Item key={item} label={item} />
        ))}
      </ul>
      <>
        <span>fragment child</span>
      </>
      <Dynamic childFlag={2}>
        <b>dynamic</b>
      </Dynamic>
    </div>
  );
}

render(<App items={["a", "b"]} />, document.getElementById("app"));
