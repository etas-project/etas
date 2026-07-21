module tests.compiler.type_system.negative.alias_cycle;

alias A = B;
alias B = A;

flow use_alias(value: A) -> A
{
    return value;
}

flow main(args: Array<string>) -> i32
{
    return 0;
}
