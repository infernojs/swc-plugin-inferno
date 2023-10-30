import A from './A';
import Store from './Store';
Store.subscribe();
const Header = styled.div`color: red`;
_c = Header;
const StyledFactory1 = styled('div')`color: hotpink`;
_c1 = StyledFactory1;
const StyledFactory2 = styled('div')({
    color: 'hotpink'
});
_c2 = StyledFactory2;
const StyledFactory3 = styled(A)({
    color: 'hotpink'
});
_c3 = StyledFactory3;
const FunnyFactory = funny.factory``;
let Alias1 = A;
let Alias2 = A.Foo;
const Dict = {};
function Foo() {
    return [
        Inferno.createElement(A),
        Inferno.createElement(B),
        Inferno.createElement(StyledFactory1),
        Inferno.createElement(StyledFactory2),
        Inferno.createElement(StyledFactory3),
        Inferno.createElement(Alias1),
        Inferno.createElement(Alias2),
        jsx(Header),
        Inferno.createElement(Dict.X)
    ];
}
_c4 = Foo;
Inferno.createContext(Store);
const B = hoc(A);
_c5 = B;
// This is currently registered as a false positive:
const NotAComponent = wow(A); // We could avoid it but it also doesn't hurt.
_c6 = NotAComponent;
var _c, _c1, _c2, _c3, _c4, _c5, _c6;
$RefreshReg$(_c, "Header");
$RefreshReg$(_c1, "StyledFactory1");
$RefreshReg$(_c2, "StyledFactory2");
$RefreshReg$(_c3, "StyledFactory3");
$RefreshReg$(_c4, "Foo");
$RefreshReg$(_c5, "B");
$RefreshReg$(_c6, "NotAComponent");
