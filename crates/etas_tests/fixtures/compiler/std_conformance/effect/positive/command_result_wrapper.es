// support: Command, CommandResult, SandboxProfile, Command.run, SandboxViolation
// layer: effect
// polarity: positive
// status: covered-positive
module tests.compiler.std_conformance.effect.command_result_wrapper;

import std.host.command.{Command, CommandResult, run};

flow run_checked(command: Command) -> CommandResult ![Command.run<DefaultCommandSandbox>] {
    return run(command, DefaultCommandSandbox);
}

flow main(args: Array<string>) -> i32 {
    return 0;
}
