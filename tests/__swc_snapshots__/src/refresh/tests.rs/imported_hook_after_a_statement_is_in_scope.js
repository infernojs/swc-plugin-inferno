'use client';
var _s = $RefreshSig$();
console.log('loaded');
import { useFancyState } from './hooks';
export function App() {
    _s();
    const bar = useFancyState();
    return <h1>{bar}</h1>;
}
_s(App, "useFancyState{bar}", false, function() {
    return [
        useFancyState
    ];
});
_c = App;
var _c;
$RefreshReg$(_c, "App");
