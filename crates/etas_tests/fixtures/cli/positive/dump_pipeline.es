module tests.phase0;

alias Draft = string;

agent Writer(input: Draft) -> Draft {}

flow main(args: Array<string>) -> i32 {
    let input = "draft";
    let draft = input ~> Writer;
    return 0;
}
