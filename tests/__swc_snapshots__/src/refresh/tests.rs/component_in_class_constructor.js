import { Component } from 'inferno';
import { useState } from 'inferno-hooks';
export class Table extends Component {
    constructor(props){
        var _s = $RefreshSig$();
        super(props);
        const Row = ()=>{
            _s();
            const [x] = useState(0);
            return <tr>{x}</tr>;
        };
        _s(Row, "useState{[x](0)}");
        this.row = Row;
    }
}
