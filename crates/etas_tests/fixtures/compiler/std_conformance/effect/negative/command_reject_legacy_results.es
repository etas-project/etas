// support: Command, CommandResult, SandboxProfile, Command.run, SandboxViolation
// layer: effect
// polarity: negative
// status: covered-negative
// expect: std.command.run returns CommandResult, not string or bare status code
// expect: sandbox argument is required and participates in Command.run<sandbox>
module tests.compiler.std_conformance.effect.command_reject_legacy_results;

import std.host.command.{Command, CommandResult, run};

flow bad_string(command: Command) -> string ![Command.run<DefaultCommandSandbox>] {
    return run(command, DefaultCommandSandbox);
}

flow bad_status(command: Command) -> i32 ![Command.run<DefaultCommandSandbox>] {
    return run(command, DefaultCommandSandbox);
}

flow missing_sandbox(command: Command) -> CommandResult ![Command.run] {
    return run(command);
}

flow bad_sandbox(command: Command) -> CommandResult ![Command.run<"raw">] {
    return run(command, "raw");
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
