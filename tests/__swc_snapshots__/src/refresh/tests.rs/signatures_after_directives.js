'use client';
var _s = $RefreshSig$();
import { useState } from 'inferno-hooks';
export function Counter() {
    'use strict';
    _s();
    const [count] = useState(0);
    function Inner() {
        'use strict';
        var _s = $RefreshSig$();
        const Row = ()=>{
            _s();
            const [x] = useState(1);
            return <b>{x}</b>;
        };
        _s(Row, "useState{[x](1)}");
        return <Row/>;
    }
    return <div>{count}<Inner/></div>;
}
_s(Counter, "useState{[count](0)}");
_c = Counter;
var _c;
$RefreshReg$(_c, "Counter");
