module examples.handler_tails;

effect Choice {
    action choose(left: i32, right: i32) -> i32;
}

flow main(args: Array<string>) -> i32 ![] {
    let first = handler {
        Choice.choose(left, right) => left;
    };
    let selected = perform Choice.choose(20, 22) with first;
    return perform Choice.choose(selected, 22) with {
        Choice.choose(left, right) => {
            if left == 0 { finish 0; }
            left + right
        }
    };
}
