module library.api;

public effect Probe { action hit<T>() -> unit; }
flow emit<T>(value: T) -> unit ![Probe.hit<T>] {
    perform Probe.hit<T>();
    return;
}

public type Container<T> = { value: T };
impl Container<T> {
    public flow capture<T>(self, other: T) -> (() -> unit) {
        return () => {
            emit(self.value);
            emit(other);
            return;
        };
    }
    public flow wrap(self) -> (() -> unit) {
        return self.capture("other");
    }
}

public flow lookup() -> i32 ![] { return 7; }
