alias Draft = string;

flow main(args: Array<string>) -> i32 {
    let input = "draft";
    let draft = input ~> MissingWriter;
    return 0;
}
