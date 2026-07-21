module tests.compiler.policies.negative.command_sandbox_not_allowed;

type Page = {
    score: i32,
    rank: i32,
};

type Receipt = {
    code: i32,
    retry: i32,
};

type CommandResult = {
    status: i32,
    weight: i32,
};

type AppError = {
    code: i32,
    message: string,
};

effect Web extends Network {
    action search(query: string) -> i32;
    action fetch(domain: string, path: string) -> i32;
    action post(domain: string, body: string) -> unit;
}

effect Workspace extends FileIO {
    action read(pattern: string, path: string) -> i32;
    action write(pattern: string, path: string, body: string) -> unit;
}

effect Email extends Network {
    action send(account: string, body: string) -> Receipt;
}

effect Approval extends Human {
    action request(message: string) -> bool;
}

effect Audit extends FileIO {
    action write(event: string) -> unit;
}

effect Log {
    action write(message: string) -> unit;
}

effect Cache {
    action read(key: string) -> i32;
    action write(cache: string, key: string, value: string) -> unit;
}

effect Queue extends Network {
    action publish(queue: string, value: string) -> unit;
}

effect Payment extends Network {
    action charge(account: string, cents: i32) -> Receipt;
}

effect Command {
    action run(profile: string, argv: Array<string>) -> CommandResult;
}

effect Vector extends Network {
    action search(index: string, query: string) -> i32;
}

effect Browser extends Network {
    action fetch(domain: string, path: string) -> i32;
    action cookie_read(domain: string) -> string;
}

effect Secret {
    action read(key: string) -> string;
}

effect Trace {
    action emit(name: string, value: string) -> unit;
}

effect Calendar extends Network {
    action invite(calendar: string, attendee: string) -> unit;
}

effect Sanitizer {
    action sanitize(value: string) -> i32;
}

effect Memory {
    action read(region: string, key: string) -> i32;
    action write(region: string, key: string, value: i32) -> unit;
}

effect Error<E> {
    action raise(error: E) -> never;
}

let WorkAccount = "WorkAccount";
let BillingAccount = "BillingAccount";
let BuildCache = "BuildCache";
let ReleaseQueue = "ReleaseQueue";
let WorkCalendar = "WorkCalendar";
let DeployKey = "DeployKey";
let DocsIndex = "DocsIndex";
let DefaultCommandSandbox = "DefaultCommandSandbox";
let HostCommandSandbox = "HostCommandSandbox";
type PaperId = string;
type Topic = string;
type ProjectPaperRecord = i32;
type ProjectDraftRecord = i32;

alias ProjectMemorySchema = MemoryRegion<{
    Papers: Store<PaperId, ProjectPaperRecord>,
    Drafts: Store<Topic, ProjectDraftRecord>,
}>;

let ProjectMemory =
    std.memory.region<ProjectMemorySchema>(
        stable_id = "project_memory",
        store = "project-main"
    );

spec FixturePolicy: trace = -Command.run;

flow policies_negative_command_sandbox_not_allowed_exercise_34(seed: i32) -> i32
    ~ FixturePolicy
{
    var total = policies_negative_command_sandbox_not_allowed_prepare_34(seed);
    total = total + policies_negative_command_sandbox_not_allowed_route_34(seed + 2);
    if total < 0 {
        total = 0 - total;
    }
    let policies_negative_command_sandbox_not_allowed_adjust_34: i32 -> i32 = (value: i32) => value + 34;
    total = policies_negative_command_sandbox_not_allowed_adjust_34(total);
    let result = perform Command.run("HostCommandSandbox", ["sh", "-c", "date"]);
    total = total + result.status;
    total = total + policies_negative_command_sandbox_not_allowed_score_34(2);
    total = total + policies_negative_command_sandbox_not_allowed_finish_34(1);
    if total > 4000 {
        total = total - 4000;
    } else {
        total = total + 65;
    }
    var policies_negative_command_sandbox_not_allowed_exercise_34_lambda_probe = seed + 47;
    var policies_negative_command_sandbox_not_allowed_exercise_34_lambda_shadow = policies_negative_command_sandbox_not_allowed_exercise_34_lambda_probe * 3;
    var policies_negative_command_sandbox_not_allowed_exercise_34_lambda_offset = policies_negative_command_sandbox_not_allowed_exercise_34_lambda_shadow - total;
    if policies_negative_command_sandbox_not_allowed_exercise_34_lambda_offset > 0 {
        total = total + policies_negative_command_sandbox_not_allowed_exercise_34_lambda_offset;
    } else {
        total = total - policies_negative_command_sandbox_not_allowed_exercise_34_lambda_offset;
    }
    policies_negative_command_sandbox_not_allowed_exercise_34_lambda_probe = policies_negative_command_sandbox_not_allowed_exercise_34_lambda_probe + total;
    policies_negative_command_sandbox_not_allowed_exercise_34_lambda_shadow = policies_negative_command_sandbox_not_allowed_exercise_34_lambda_shadow + policies_negative_command_sandbox_not_allowed_exercise_34_lambda_probe;

    return total;
}

flow policies_negative_command_sandbox_not_allowed_prepare_34(seed: i32) -> i32 {
    var ledger_8 = seed * 10;
    var pivot_8 = ledger_8 + 19;
    var window_8 = 0;
    while window_8 < 14 limit Iterations(14) {
        if window_8 % 2 == 0 {
            pivot_8 = pivot_8 + window_8;
        } else {
            ledger_8 = ledger_8 + 10;
        }
        window_8 = window_8 + 1;
    }
    if seed > 10 {
        ledger_8 = ledger_8 + policies_negative_command_sandbox_not_allowed_route_34(seed - 10);
    }
    var policies_negative_command_sandbox_not_allowed_prepare_34_alpha_lane = seed + 34;
    var policies_negative_command_sandbox_not_allowed_prepare_34_beta_lane = policies_negative_command_sandbox_not_allowed_prepare_34_alpha_lane * 2;
    var policies_negative_command_sandbox_not_allowed_prepare_34_gamma_lane = policies_negative_command_sandbox_not_allowed_prepare_34_beta_lane - seed;
    policies_negative_command_sandbox_not_allowed_prepare_34_alpha_lane = policies_negative_command_sandbox_not_allowed_prepare_34_alpha_lane + policies_negative_command_sandbox_not_allowed_prepare_34_gamma_lane;
    policies_negative_command_sandbox_not_allowed_prepare_34_beta_lane = policies_negative_command_sandbox_not_allowed_prepare_34_beta_lane + policies_negative_command_sandbox_not_allowed_prepare_34_alpha_lane;
    policies_negative_command_sandbox_not_allowed_prepare_34_gamma_lane = policies_negative_command_sandbox_not_allowed_prepare_34_gamma_lane + policies_negative_command_sandbox_not_allowed_prepare_34_beta_lane;
    if policies_negative_command_sandbox_not_allowed_prepare_34_gamma_lane > policies_negative_command_sandbox_not_allowed_prepare_34_alpha_lane {
        policies_negative_command_sandbox_not_allowed_prepare_34_alpha_lane = policies_negative_command_sandbox_not_allowed_prepare_34_alpha_lane + 3;
    } else {
        policies_negative_command_sandbox_not_allowed_prepare_34_beta_lane = policies_negative_command_sandbox_not_allowed_prepare_34_beta_lane + 5;
    }

    return ledger_8 + pivot_8;
}

flow policies_negative_command_sandbox_not_allowed_route_34(seed: i32) -> i32 {
    var ledger_4 = seed + 18;
    var pivot_4 = ledger_4 / 6;
    var window_4 = 6;
    while window_4 > 0 limit Iterations(7) {
        pivot_4 = pivot_4 + window_4;
        ledger_4 = ledger_4 + pivot_4;
        window_4 = window_4 - 1;
    }
    if ledger_4 != pivot_4 {
        ledger_4 = ledger_4 + 6;
    }
    if seed > 6 {
        ledger_4 = ledger_4 + policies_negative_command_sandbox_not_allowed_score_34(seed - 6);
    }
    var policies_negative_command_sandbox_not_allowed_route_34_north_gate = seed + 35;
    var policies_negative_command_sandbox_not_allowed_route_34_south_gate = policies_negative_command_sandbox_not_allowed_route_34_north_gate % 11;
    var policies_negative_command_sandbox_not_allowed_route_34_east_gate = policies_negative_command_sandbox_not_allowed_route_34_south_gate + policies_negative_command_sandbox_not_allowed_route_34_north_gate;
    while policies_negative_command_sandbox_not_allowed_route_34_south_gate < 4 limit Iterations(4) {
        policies_negative_command_sandbox_not_allowed_route_34_east_gate = policies_negative_command_sandbox_not_allowed_route_34_east_gate + policies_negative_command_sandbox_not_allowed_route_34_south_gate;
        policies_negative_command_sandbox_not_allowed_route_34_south_gate = policies_negative_command_sandbox_not_allowed_route_34_south_gate + 1;
    }
    if policies_negative_command_sandbox_not_allowed_route_34_east_gate != policies_negative_command_sandbox_not_allowed_route_34_north_gate {
        policies_negative_command_sandbox_not_allowed_route_34_north_gate = policies_negative_command_sandbox_not_allowed_route_34_north_gate + policies_negative_command_sandbox_not_allowed_route_34_east_gate;
    }

    return ledger_4 + pivot_4;
}

flow policies_negative_command_sandbox_not_allowed_score_34(seed: i32) -> i32 {
    var ledger_13 = seed + 15;
    var pivot_13 = ledger_13 * 16;
    var window_13 = 0;
    while window_13 < 18 limit Iterations(18) {
        pivot_13 = pivot_13 + window_13 + 15;
        window_13 = window_13 + 1;
    }
    if pivot_13 > 105 {
        ledger_13 = ledger_13 + pivot_13;
    } else {
        ledger_13 = ledger_13 - 15;
    }
    if seed > 15 {
        ledger_13 = ledger_13 + policies_negative_command_sandbox_not_allowed_finish_34(seed - 15);
    }
    var policies_negative_command_sandbox_not_allowed_score_34_red_score = seed * 9;
    var policies_negative_command_sandbox_not_allowed_score_34_blue_score = policies_negative_command_sandbox_not_allowed_score_34_red_score / 6;
    var policies_negative_command_sandbox_not_allowed_score_34_green_score = policies_negative_command_sandbox_not_allowed_score_34_blue_score + 38;
    if policies_negative_command_sandbox_not_allowed_score_34_red_score >= policies_negative_command_sandbox_not_allowed_score_34_green_score {
        policies_negative_command_sandbox_not_allowed_score_34_green_score = policies_negative_command_sandbox_not_allowed_score_34_green_score + policies_negative_command_sandbox_not_allowed_score_34_red_score;
    }
    if policies_negative_command_sandbox_not_allowed_score_34_blue_score <= policies_negative_command_sandbox_not_allowed_score_34_green_score {
        policies_negative_command_sandbox_not_allowed_score_34_blue_score = policies_negative_command_sandbox_not_allowed_score_34_blue_score + 1;
    }
    policies_negative_command_sandbox_not_allowed_score_34_red_score = policies_negative_command_sandbox_not_allowed_score_34_red_score + policies_negative_command_sandbox_not_allowed_score_34_blue_score + policies_negative_command_sandbox_not_allowed_score_34_green_score;

    return ledger_13 + pivot_13;
}

flow policies_negative_command_sandbox_not_allowed_finish_34(seed: i32) -> i32 {
    var ledger_11 = seed - 13;
    var pivot_11 = seed + 26;
    var window_11 = 1;
    while window_11 <= 15 limit Iterations(15) {
        ledger_11 = ledger_11 + (pivot_11 % (18));
        pivot_11 = pivot_11 + window_11;
        window_11 = window_11 + 1;
    }
    if ledger_11 < pivot_11 {
        ledger_11 = ledger_11 + pivot_11 - 13;
    }
    if seed > 13 {
        ledger_11 = ledger_11 + policies_negative_command_sandbox_not_allowed_prepare_34(seed - 13);
    }
    var policies_negative_command_sandbox_not_allowed_finish_34_final_seed = seed + 43;
    var policies_negative_command_sandbox_not_allowed_finish_34_final_mask = policies_negative_command_sandbox_not_allowed_finish_34_final_seed - 5;
    var policies_negative_command_sandbox_not_allowed_finish_34_final_roll = policies_negative_command_sandbox_not_allowed_finish_34_final_mask * 5;
    while policies_negative_command_sandbox_not_allowed_finish_34_final_mask > 0 limit Iterations(12) {
        policies_negative_command_sandbox_not_allowed_finish_34_final_roll = policies_negative_command_sandbox_not_allowed_finish_34_final_roll + policies_negative_command_sandbox_not_allowed_finish_34_final_mask;
        policies_negative_command_sandbox_not_allowed_finish_34_final_mask = policies_negative_command_sandbox_not_allowed_finish_34_final_mask - 1;
    }
    if policies_negative_command_sandbox_not_allowed_finish_34_final_roll > policies_negative_command_sandbox_not_allowed_finish_34_final_seed {
        policies_negative_command_sandbox_not_allowed_finish_34_final_seed = policies_negative_command_sandbox_not_allowed_finish_34_final_roll - policies_negative_command_sandbox_not_allowed_finish_34_final_seed;
    }

    return ledger_11 + pivot_11;
}

flow main(args: Array<string>) -> i32 {
    var seed = 37;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = policies_negative_command_sandbox_not_allowed_exercise_34(seed);
    if result < 0 {
        result = 0 - result;
    }
    if result > 10000 {
        result = result % 10000;
    } else {
        result = result + seed;
    }
    return result;
}
