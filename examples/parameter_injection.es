module app.parameter_injection;

import std.effects.Inject;

effect Project {
    action name() -> string;
}

flow greeting(who: string, @inject prefix: string, punctuation: string = "!",
    @inject(Project.name) project: string) -> string ![] {
    return prefix + who + " from " + project + punctuation;
}

flow select<T>(seed: T, @inject value: T) -> T ![] {
    return value;
}

type Box<T> = { seed: T };
impl Box<T> {
    flow read(self, @inject value: T) -> T ![] {
        return value;
    }
}

spec Value<A> {
    flow read(self, @inject value: A) -> A ![];
}
type ValueBox<B> = { seed: B };
impl ValueBox<B> ~ Value<B> {
    flow read(self, value: B) -> B ![] {
        return value;
    }
}

flow main(args: Array<string>) -> i32 ![] {
    let text = greeting("Etas") with {
        Inject<string>.get(name) => "Hello ";
        Project.name() => "scoped injection";
    };
    let selected = select(0) with { Inject<i32>.get(name) => 42; };
    let owner = Box { seed = 0 }.read() with { Inject<i32>.get(name) => 21; };
    let inherited = ValueBox { seed = 0 }.read() with { Inject<i32>.get(name) => 21; };
    let explicit = greeting("Etas", prefix = "Hi ", project = "explicit arguments");
    if selected == 42 && owner + inherited == 42 && text == "Hello Etas from scoped injection!" && explicit == "Hi Etas from explicit arguments!" {
        return 0;
    }
    return 1;
}
