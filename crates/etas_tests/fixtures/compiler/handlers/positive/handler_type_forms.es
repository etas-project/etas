module tests.compiler.handlers.positive.handler_type_forms;

effect Gate {
    action request(message: string) -> bool;
}

alias HandleOnly = ![Gate];
alias HandleFor = ![Gate for string];
alias Transform = ![Gate => Console.stdout_write];
alias TransformFor = ![Gate => Console.stdout_write, Error<IOError> for Result<string, IOError>];
alias EmptyProduced = ![Gate => []];

flow main(args: Array<string>) -> i32 ![]
{
    return 0;
}
