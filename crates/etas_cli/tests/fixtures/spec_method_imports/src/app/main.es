module app.main;
import library.{Reader, Item};

flow use<T ~ Reader>(value: T) -> i32 ![] {
    return value.read();
}
flow main(args: Array<string>) -> i32 ![] {
    return use(Item {});
}
