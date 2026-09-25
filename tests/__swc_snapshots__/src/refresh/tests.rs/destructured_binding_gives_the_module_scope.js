var _s = $RefreshSig$();
export const { theme } = globalThis.config;
function useTheme() {
    return theme;
}
export function App() {
    _s();
    const t = useTheme();
    return <h1>{t}</h1>;
}
_s(App, "useTheme{t}", false, function() {
    return [
        useTheme
    ];
});
_c = App;
var _c;
$RefreshReg$(_c, "App");
