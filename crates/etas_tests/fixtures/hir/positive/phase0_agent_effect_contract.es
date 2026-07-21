module tests.phase0;

alias Draft = string;

@model(model = "test-model")
agent Writer(input: Draft) -> Draft ![] {
    return Prompt.new().data(input);
}

flow main(args: Array<string>) -> i32 {
    let input = "draft";
    let draft = input ~> Writer;
    return 0;
}
