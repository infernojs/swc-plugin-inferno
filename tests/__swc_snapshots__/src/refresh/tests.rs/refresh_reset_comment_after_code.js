var _s = $RefreshSig$();
import { useState } from 'inferno-hooks';
export function Counter() {
    _s();
    const [count] = useState(0); // @refresh reset
    return <div>{count}</div>;
}
_s(Counter, "useState{[count](0)}", true);
_c = Counter;
var _c;
$RefreshReg$(_c, "Counter");
