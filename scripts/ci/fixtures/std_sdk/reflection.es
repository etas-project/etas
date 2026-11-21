module app.reflection;
import std.text.join;

flow answer() -> i32 ![] {
    return 42;
}

flow main(args: Array<string>) -> i32 ![] {
    let definition = ^answer;
    let standard = ^join;
    return if definition.id == (^answer).id
        && standard.revision == (^join).revision
        && standard.dependencies == (^join).dependencies {
        answer()
    } else {
        0
    };
}
