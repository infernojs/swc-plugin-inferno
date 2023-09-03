import Inferno from "inferno";
import Inferno from "inferno";
import { Button, Input } from "antd";
import Child from "./component/Child";

class Page extends Inferno.Component {
    render() {
        return (
            <div className={"test"}>
                <div>Page</div>
                <Child />
                <input placeholder="我是谁?" />
                <Button>click me</Button>
                <Input />
            </div>
        );
    }
}
