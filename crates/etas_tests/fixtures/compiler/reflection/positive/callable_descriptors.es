module tests.reflection;
import std.compiler.CallableDef;

flow identity(value: i32) -> i32 { return value; }
agent Worker(value: i32) -> i32 { return value; }
impl agent Worker {
    public flow extra(value: i32) -> i32 { return value + 1; }
}

flow verify() -> bool {
    let descriptors: Array<CallableDef> = [^identity, ^Worker.extra];
    let erased: CallableDef = ^identity;
    return descriptors.len() == 2
        && erased.revision == (^identity).revision
        && erased.dependencies == (^identity).dependencies
        && (^Worker).methods.len() == 1
        && match (^Worker).signature {
            Some(signature) => signature == (^Worker.run).signature,
            None => false,
        };
}

flow main(args: Array<string>) -> i32 {
    if verify() { return 0; }
    return 1;
}
