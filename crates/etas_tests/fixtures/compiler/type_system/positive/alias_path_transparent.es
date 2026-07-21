module tests.compiler.type_system.positive.alias_path_transparent;

alias Path = string;

flow accept_string(value: string) -> string
{
    return value;
}

flow accept_path(value: Path) -> Path
{
    return value;
}

flow main(args: Array<string>) -> i32
{
    let raw: string = "/workspace/input.es";
    let path: Path = raw;
    let round_trip: string = accept_string(path);
    let second: Path = accept_path(round_trip);
    let third: string = second;
    return 0;
}
