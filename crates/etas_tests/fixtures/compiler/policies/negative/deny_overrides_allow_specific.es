module tests.compiler.policies.negative.deny_overrides_allow_specific;

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

spec FixturePolicy: trace = +Browser.fetch & -Browser;

flow policies_negative_deny_overrides_allow_specific_exercise_35(seed: i32) -> i32
    ~ FixturePolicy
{
    var total = policies_negative_deny_overrides_allow_specific_prepare_35(seed);
    total = total + policies_negative_deny_overrides_allow_specific_route_35(seed + 2);
    if total < 0 {
        total = 0 - total;
    }
    let policies_negative_deny_overrides_allow_specific_adjust_35: i32 -> i32 = (value: i32) => value + 35;
    total = policies_negative_deny_overrides_allow_specific_adjust_35(total);
    total = total + perform Browser.fetch("example.com", "/");
    total = total + policies_negative_deny_overrides_allow_specific_score_35(2);
    total = total + policies_negative_deny_overrides_allow_specific_finish_35(1);
    if total > 4000 {
        total = total - 4000;
    } else {
        total = total + 66;
    }
    var policies_negative_deny_overrides_allow_specific_exercise_35_lambda_probe = seed + 48;
    var policies_negative_deny_overrides_allow_specific_exercise_35_lambda_shadow = policies_negative_deny_overrides_allow_specific_exercise_35_lambda_probe * 3;
    var policies_negative_deny_overrides_allow_specific_exercise_35_lambda_offset = policies_negative_deny_overrides_allow_specific_exercise_35_lambda_shadow - total;
    if policies_negative_deny_overrides_allow_specific_exercise_35_lambda_offset > 0 {
        total = total + policies_negative_deny_overrides_allow_specific_exercise_35_lambda_offset;
    } else {
        total = total - policies_negative_deny_overrides_allow_specific_exercise_35_lambda_offset;
    }
    policies_negative_deny_overrides_allow_specific_exercise_35_lambda_probe = policies_negative_deny_overrides_allow_specific_exercise_35_lambda_probe + total;
    policies_negative_deny_overrides_allow_specific_exercise_35_lambda_shadow = policies_negative_deny_overrides_allow_specific_exercise_35_lambda_shadow + policies_negative_deny_overrides_allow_specific_exercise_35_lambda_probe;

    return total;
}

flow policies_negative_deny_overrides_allow_specific_prepare_35(seed: i32) -> i32 {
    var ledger_9 = seed + 33;
    var pivot_9 = ledger_9 / 11;
    var window_9 = 11;
    while window_9 > 0 limit Iterations(12) {
        pivot_9 = pivot_9 + window_9;
        ledger_9 = ledger_9 + pivot_9;
        window_9 = window_9 - 1;
    }
    if ledger_9 != pivot_9 {
        ledger_9 = ledger_9 + 11;
    }
    if seed > 11 {
        ledger_9 = ledger_9 + policies_negative_deny_overrides_allow_specific_route_35(seed - 11);
    }
    var policies_negative_deny_overrides_allow_specific_prepare_35_alpha_lane = seed + 35;
    var policies_negative_deny_overrides_allow_specific_prepare_35_beta_lane = policies_negative_deny_overrides_allow_specific_prepare_35_alpha_lane * 2;
    var policies_negative_deny_overrides_allow_specific_prepare_35_gamma_lane = policies_negative_deny_overrides_allow_specific_prepare_35_beta_lane - seed;
    policies_negative_deny_overrides_allow_specific_prepare_35_alpha_lane = policies_negative_deny_overrides_allow_specific_prepare_35_alpha_lane + policies_negative_deny_overrides_allow_specific_prepare_35_gamma_lane;
    policies_negative_deny_overrides_allow_specific_prepare_35_beta_lane = policies_negative_deny_overrides_allow_specific_prepare_35_beta_lane + policies_negative_deny_overrides_allow_specific_prepare_35_alpha_lane;
    policies_negative_deny_overrides_allow_specific_prepare_35_gamma_lane = policies_negative_deny_overrides_allow_specific_prepare_35_gamma_lane + policies_negative_deny_overrides_allow_specific_prepare_35_beta_lane;
    if policies_negative_deny_overrides_allow_specific_prepare_35_gamma_lane > policies_negative_deny_overrides_allow_specific_prepare_35_alpha_lane {
        policies_negative_deny_overrides_allow_specific_prepare_35_alpha_lane = policies_negative_deny_overrides_allow_specific_prepare_35_alpha_lane + 3;
    } else {
        policies_negative_deny_overrides_allow_specific_prepare_35_beta_lane = policies_negative_deny_overrides_allow_specific_prepare_35_beta_lane + 5;
    }

    return ledger_9 + pivot_9;
}

flow policies_negative_deny_overrides_allow_specific_route_35(seed: i32) -> i32 {
    var ledger_5 = seed + 7;
    var pivot_5 = ledger_5 * 8;
    var window_5 = 0;
    while window_5 < 10 limit Iterations(10) {
        pivot_5 = pivot_5 + window_5 + 7;
        window_5 = window_5 + 1;
    }
    if pivot_5 > 49 {
        ledger_5 = ledger_5 + pivot_5;
    } else {
        ledger_5 = ledger_5 - 7;
    }
    if seed > 7 {
        ledger_5 = ledger_5 + policies_negative_deny_overrides_allow_specific_score_35(seed - 7);
    }
    var policies_negative_deny_overrides_allow_specific_route_35_north_gate = seed + 36;
    var policies_negative_deny_overrides_allow_specific_route_35_south_gate = policies_negative_deny_overrides_allow_specific_route_35_north_gate % 5;
    var policies_negative_deny_overrides_allow_specific_route_35_east_gate = policies_negative_deny_overrides_allow_specific_route_35_south_gate + policies_negative_deny_overrides_allow_specific_route_35_north_gate;
    while policies_negative_deny_overrides_allow_specific_route_35_south_gate < 4 limit Iterations(4) {
        policies_negative_deny_overrides_allow_specific_route_35_east_gate = policies_negative_deny_overrides_allow_specific_route_35_east_gate + policies_negative_deny_overrides_allow_specific_route_35_south_gate;
        policies_negative_deny_overrides_allow_specific_route_35_south_gate = policies_negative_deny_overrides_allow_specific_route_35_south_gate + 1;
    }
    if policies_negative_deny_overrides_allow_specific_route_35_east_gate != policies_negative_deny_overrides_allow_specific_route_35_north_gate {
        policies_negative_deny_overrides_allow_specific_route_35_north_gate = policies_negative_deny_overrides_allow_specific_route_35_north_gate + policies_negative_deny_overrides_allow_specific_route_35_east_gate;
    }

    return ledger_5 + pivot_5;
}

flow policies_negative_deny_overrides_allow_specific_score_35(seed: i32) -> i32 {
    var ledger_14 = seed - 16;
    var pivot_14 = seed + 32;
    var window_14 = 1;
    while window_14 <= 18 limit Iterations(18) {
        ledger_14 = ledger_14 + (pivot_14 % (21));
        pivot_14 = pivot_14 + window_14;
        window_14 = window_14 + 1;
    }
    if ledger_14 < pivot_14 {
        ledger_14 = ledger_14 + pivot_14 - 16;
    }
    if seed > 16 {
        ledger_14 = ledger_14 + policies_negative_deny_overrides_allow_specific_finish_35(seed - 16);
    }
    var policies_negative_deny_overrides_allow_specific_score_35_red_score = seed * 10;
    var policies_negative_deny_overrides_allow_specific_score_35_blue_score = policies_negative_deny_overrides_allow_specific_score_35_red_score / 2;
    var policies_negative_deny_overrides_allow_specific_score_35_green_score = policies_negative_deny_overrides_allow_specific_score_35_blue_score + 39;
    if policies_negative_deny_overrides_allow_specific_score_35_red_score >= policies_negative_deny_overrides_allow_specific_score_35_green_score {
        policies_negative_deny_overrides_allow_specific_score_35_green_score = policies_negative_deny_overrides_allow_specific_score_35_green_score + policies_negative_deny_overrides_allow_specific_score_35_red_score;
    }
    if policies_negative_deny_overrides_allow_specific_score_35_blue_score <= policies_negative_deny_overrides_allow_specific_score_35_green_score {
        policies_negative_deny_overrides_allow_specific_score_35_blue_score = policies_negative_deny_overrides_allow_specific_score_35_blue_score + 1;
    }
    policies_negative_deny_overrides_allow_specific_score_35_red_score = policies_negative_deny_overrides_allow_specific_score_35_red_score + policies_negative_deny_overrides_allow_specific_score_35_blue_score + policies_negative_deny_overrides_allow_specific_score_35_green_score;

    return ledger_14 + pivot_14;
}

flow policies_negative_deny_overrides_allow_specific_finish_35(seed: i32) -> i32 {
    var ledger_12 = seed * 14;
    var pivot_12 = ledger_12 + 23;
    var window_12 = 0;
    while window_12 < 18 limit Iterations(18) {
        if window_12 % 2 == 0 {
            pivot_12 = pivot_12 + window_12;
        } else {
            ledger_12 = ledger_12 + 14;
        }
        window_12 = window_12 + 1;
    }
    if seed > 14 {
        ledger_12 = ledger_12 + policies_negative_deny_overrides_allow_specific_prepare_35(seed - 14);
    }
    var policies_negative_deny_overrides_allow_specific_finish_35_final_seed = seed + 44;
    var policies_negative_deny_overrides_allow_specific_finish_35_final_mask = policies_negative_deny_overrides_allow_specific_finish_35_final_seed - 6;
    var policies_negative_deny_overrides_allow_specific_finish_35_final_roll = policies_negative_deny_overrides_allow_specific_finish_35_final_mask * 6;
    while policies_negative_deny_overrides_allow_specific_finish_35_final_mask > 0 limit Iterations(12) {
        policies_negative_deny_overrides_allow_specific_finish_35_final_roll = policies_negative_deny_overrides_allow_specific_finish_35_final_roll + policies_negative_deny_overrides_allow_specific_finish_35_final_mask;
        policies_negative_deny_overrides_allow_specific_finish_35_final_mask = policies_negative_deny_overrides_allow_specific_finish_35_final_mask - 1;
    }
    if policies_negative_deny_overrides_allow_specific_finish_35_final_roll > policies_negative_deny_overrides_allow_specific_finish_35_final_seed {
        policies_negative_deny_overrides_allow_specific_finish_35_final_seed = policies_negative_deny_overrides_allow_specific_finish_35_final_roll - policies_negative_deny_overrides_allow_specific_finish_35_final_seed;
    }

    return ledger_12 + pivot_12;
}

flow main(args: Array<string>) -> i32 {
    var seed = 38;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = policies_negative_deny_overrides_allow_specific_exercise_35(seed);
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
