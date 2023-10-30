var _s = $RefreshSig$(), _s1 = $RefreshSig$();
export const A = _s(Inferno.memo(_c1 = _s(Inferno.forwardRef(_c = _s((props, ref1)=>{
    _s();
    const [foo, setFoo] = useState(0);
    Inferno.useEffect(()=>{});
    return <h1 ref={ref1}>{foo}</h1>;
}, "useState{[foo, setFoo](0)}\nuseEffect{}")), "useState{[foo, setFoo](0)}\nuseEffect{}")), "useState{[foo, setFoo](0)}\nuseEffect{}");
_c2 = A;
export const B = _s1(Inferno.memo(_c4 = _s1(Inferno.forwardRef(_c3 = _s1(function(props, ref1) {
    _s1();
    const [foo, setFoo] = useState(0);
    Inferno.useEffect(()=>{});
    return <h1 ref={ref1}>{foo}</h1>;
}, "useState{[foo, setFoo](0)}\nuseEffect{}")), "useState{[foo, setFoo](0)}\nuseEffect{}")), "useState{[foo, setFoo](0)}\nuseEffect{}");
_c5 = B;
function hoc() {
    var _s = $RefreshSig$();
    return _s(function Inner() {
        _s();
        const [foo, setFoo] = useState(0);
        Inferno.useEffect(()=>{});
        return <h1 ref={ref}>{foo}</h1>;
    }, "useState{[foo, setFoo](0)}\nuseEffect{}");
}
export let C = hoc();
var _c, _c1, _c2, _c3, _c4, _c5;
$RefreshReg$(_c, "A$Inferno.memo$Inferno.forwardRef");
$RefreshReg$(_c1, "A$Inferno.memo");
$RefreshReg$(_c2, "A");
$RefreshReg$(_c3, "B$Inferno.memo$Inferno.forwardRef");
$RefreshReg$(_c4, "B$Inferno.memo");
$RefreshReg$(_c5, "B");
