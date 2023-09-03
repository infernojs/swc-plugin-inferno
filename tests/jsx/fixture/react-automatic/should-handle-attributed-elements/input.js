var HelloMessage = Inferno.createClass({
    render: function () {
        return <div>Hello {this.props.name}</div>;
    },
});

Inferno.render(<HelloMessage name={<span>Sebastian</span>} />, mountNode);
