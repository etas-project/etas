module library.api;

public effect Probe { action hit<T>() -> unit; }
public spec Read<T> { flow read(self) -> T ![Probe.hit<T>]; }
public type Container<T> = { value: T };
impl Container<T> ~ Read<T> {
    public flow read(self) -> T ![Probe.hit<T>] {
        perform Probe.hit<T>();
        return self.value;
    }
}

public flow load<S ~ Read<i32>>(value: S) -> i32 ![Probe.hit<i32>] {
    return value.read();
}

public flow lookup() -> i32 ![] { return 7; }
