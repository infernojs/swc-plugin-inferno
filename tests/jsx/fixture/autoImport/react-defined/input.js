import * as inferno from "inferno";
var y = inferno.createElement("div", { foo: 1 });
var x = (
    <div>
        <div key="1" />
        <div key="2" meow="wolf" />
        <div key="3" />
        <div {...props} key="4" />
    </div>
);
