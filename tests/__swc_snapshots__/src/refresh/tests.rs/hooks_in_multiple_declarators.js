var _s = $RefreshSig$();
import { useA, useB, useC } from './hooks';
export function App() {
    _s();
    const a = useA(), b = 1;
    const c = f(useB()), d = f(useC());
    return <div>{a}{b}{c}{d}</div>;
}
_s(App, "useA{a}\nuseB{}\nuseC{}", false, function() {
    return [
        useA,
        useB,
        useC
    ];
});
_c = App;
var _c;
$RefreshReg$(_c, "App");
