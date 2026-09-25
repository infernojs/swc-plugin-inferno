var _s = $RefreshSig$();
import { useState } from 'inferno-hooks';
export function useCounter(depth) {
    _s();
    const [count] = useState(0);
    return depth > 0 ? useCounter(depth - 1) : count;
}
_s(useCounter, "useState{[count](0)}\nuseCounter{}");
