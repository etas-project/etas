module library.wiring;
public effect Supply {
    action first() -> i32;
    action value<T>() -> T;
}
public type Settings = {
    @inject(Supply.first)
    first: i32,
    explicit: i32,
    @inject(Supply.first)
    second: i32,
};
public type Box<T> = {
    marker: T,
    @inject
    value: T,
};
public type NamedBox<T> = {
    marker: T,
    @inject(Supply.value<T>)
    value: T,
};
public alias NumberKey = i32;
public alias IntBox = Box<i32>;
public agent Counter = {
    @inject
    n: i32,
};
impl Counter {
    public flow get(self) -> i32 ![] { return self.n; }
    public flow increment(var self) -> i32 ![] {
        self.n = self.n + 1;
        return self.n;
    }
}

public agent Echo = {};
impl Echo {
    public flow run<T>(self, value: T) -> T {
        return value;
    }
}
public alias EchoStage = Echo;
public type Increment = {};
impl Increment {
    public flow run(self, value: i32) -> i32 {
        return value + 1;
    }
}

public agent Ticker = { n: i32 };
impl Ticker ~ std.agent.Autonomous {
    flow main(var self) -> unit {
        self.n = self.n + 1;
        return;
    }
}
impl Ticker {
    public flow barrier(var self) -> i32 ![] {
        return self.n;
    }
    public flow get(self) -> i32 ![] {
        return self.n;
    }
}
