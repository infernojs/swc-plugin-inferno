// Foo.jsx
import Inferno from "inferno";

export default function Foo() {
    return (
        <div
            onClick={async (e) => {
                await doSomething();
            }}
        ></div>
    );
}

Foo.displayName = "Foo";
