module tests.compiler.policies.negative.tool_policy_missing_sanitizer;

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

spec FixturePolicy: trace = +Web.search & (Sanitizer.sanitize << Web.search);

flow policies_negative_tool_policy_missing_sanitizer_exercise_44(seed: i32) -> i32
    ~ FixturePolicy
{
    var total = policies_negative_tool_policy_missing_sanitizer_prepare_44(seed);
    total = total + policies_negative_tool_policy_missing_sanitizer_route_44(seed + 2);
    if total < 0 {
        total = 0 - total;
    }
    let policies_negative_tool_policy_missing_sanitizer_adjust_44: i32 -> i32 = (value: i32) => value + 44;
    total = policies_negative_tool_policy_missing_sanitizer_adjust_44(total);
    total = total + perform Web.search("unsafe search");
    total = total + policies_negative_tool_policy_missing_sanitizer_score_44(2);
    total = total + policies_negative_tool_policy_missing_sanitizer_finish_44(1);
    if total > 4000 {
        total = total - 4000;
    } else {
        total = total + 75;
    }
    var policies_negative_tool_policy_missing_sanitizer_exercise_44_lambda_probe = seed + 57;
    var policies_negative_tool_policy_missing_sanitizer_exercise_44_lambda_shadow = policies_negative_tool_policy_missing_sanitizer_exercise_44_lambda_probe * 3;
    var policies_negative_tool_policy_missing_sanitizer_exercise_44_lambda_offset = policies_negative_tool_policy_missing_sanitizer_exercise_44_lambda_shadow - total;
    if policies_negative_tool_policy_missing_sanitizer_exercise_44_lambda_offset > 0 {
        total = total + policies_negative_tool_policy_missing_sanitizer_exercise_44_lambda_offset;
    } else {
        total = total - policies_negative_tool_policy_missing_sanitizer_exercise_44_lambda_offset;
    }
    policies_negative_tool_policy_missing_sanitizer_exercise_44_lambda_probe = policies_negative_tool_policy_missing_sanitizer_exercise_44_lambda_probe + total;
    policies_negative_tool_policy_missing_sanitizer_exercise_44_lambda_shadow = policies_negative_tool_policy_missing_sanitizer_exercise_44_lambda_shadow + policies_negative_tool_policy_missing_sanitizer_exercise_44_lambda_probe;

    return total;
}

flow policies_negative_tool_policy_missing_sanitizer_prepare_44(seed: i32) -> i32 {
    var ledger_9 = seed + 11;
    var pivot_9 = ledger_9 * 12;
    var window_9 = 0;
    while window_9 < 14 limit Iterations(14) {
        pivot_9 = pivot_9 + window_9 + 11;
        window_9 = window_9 + 1;
    }
    if pivot_9 > 77 {
        ledger_9 = ledger_9 + pivot_9;
    } else {
        ledger_9 = ledger_9 - 11;
    }
    if seed > 11 {
        ledger_9 = ledger_9 + policies_negative_tool_policy_missing_sanitizer_route_44(seed - 11);
    }
    var policies_negative_tool_policy_missing_sanitizer_prepare_44_alpha_lane = seed + 44;
    var policies_negative_tool_policy_missing_sanitizer_prepare_44_beta_lane = policies_negative_tool_policy_missing_sanitizer_prepare_44_alpha_lane * 2;
    var policies_negative_tool_policy_missing_sanitizer_prepare_44_gamma_lane = policies_negative_tool_policy_missing_sanitizer_prepare_44_beta_lane - seed;
    policies_negative_tool_policy_missing_sanitizer_prepare_44_alpha_lane = policies_negative_tool_policy_missing_sanitizer_prepare_44_alpha_lane + policies_negative_tool_policy_missing_sanitizer_prepare_44_gamma_lane;
    policies_negative_tool_policy_missing_sanitizer_prepare_44_beta_lane = policies_negative_tool_policy_missing_sanitizer_prepare_44_beta_lane + policies_negative_tool_policy_missing_sanitizer_prepare_44_alpha_lane;
    policies_negative_tool_policy_missing_sanitizer_prepare_44_gamma_lane = policies_negative_tool_policy_missing_sanitizer_prepare_44_gamma_lane + policies_negative_tool_policy_missing_sanitizer_prepare_44_beta_lane;
    if policies_negative_tool_policy_missing_sanitizer_prepare_44_gamma_lane > policies_negative_tool_policy_missing_sanitizer_prepare_44_alpha_lane {
        policies_negative_tool_policy_missing_sanitizer_prepare_44_alpha_lane = policies_negative_tool_policy_missing_sanitizer_prepare_44_alpha_lane + 3;
    } else {
        policies_negative_tool_policy_missing_sanitizer_prepare_44_beta_lane = policies_negative_tool_policy_missing_sanitizer_prepare_44_beta_lane + 5;
    }

    return ledger_9 + pivot_9;
}

flow policies_negative_tool_policy_missing_sanitizer_route_44(seed: i32) -> i32 {
    var ledger_3 = seed - 5;
    var pivot_3 = seed + 10;
    var window_3 = 1;
    while window_3 <= 7 limit Iterations(7) {
        ledger_3 = ledger_3 + (pivot_3 % (10));
        pivot_3 = pivot_3 + window_3;
        window_3 = window_3 + 1;
    }
    if ledger_3 < pivot_3 {
        ledger_3 = ledger_3 + pivot_3 - 5;
    }
    if seed > 5 {
        ledger_3 = ledger_3 + policies_negative_tool_policy_missing_sanitizer_score_44(seed - 5);
    }
    var policies_negative_tool_policy_missing_sanitizer_route_44_north_gate = seed + 45;
    var policies_negative_tool_policy_missing_sanitizer_route_44_south_gate = policies_negative_tool_policy_missing_sanitizer_route_44_north_gate % 7;
    var policies_negative_tool_policy_missing_sanitizer_route_44_east_gate = policies_negative_tool_policy_missing_sanitizer_route_44_south_gate + policies_negative_tool_policy_missing_sanitizer_route_44_north_gate;
    while policies_negative_tool_policy_missing_sanitizer_route_44_south_gate < 4 limit Iterations(4) {
        policies_negative_tool_policy_missing_sanitizer_route_44_east_gate = policies_negative_tool_policy_missing_sanitizer_route_44_east_gate + policies_negative_tool_policy_missing_sanitizer_route_44_south_gate;
        policies_negative_tool_policy_missing_sanitizer_route_44_south_gate = policies_negative_tool_policy_missing_sanitizer_route_44_south_gate + 1;
    }
    if policies_negative_tool_policy_missing_sanitizer_route_44_east_gate != policies_negative_tool_policy_missing_sanitizer_route_44_north_gate {
        policies_negative_tool_policy_missing_sanitizer_route_44_north_gate = policies_negative_tool_policy_missing_sanitizer_route_44_north_gate + policies_negative_tool_policy_missing_sanitizer_route_44_east_gate;
    }

    return ledger_3 + pivot_3;
}

flow policies_negative_tool_policy_missing_sanitizer_score_44(seed: i32) -> i32 {
    var ledger_10 = seed * 12;
    var pivot_10 = ledger_10 + 21;
    var window_10 = 0;
    while window_10 < 16 limit Iterations(16) {
        if window_10 % 2 == 0 {
            pivot_10 = pivot_10 + window_10;
        } else {
            ledger_10 = ledger_10 + 12;
        }
        window_10 = window_10 + 1;
    }
    if seed > 12 {
        ledger_10 = ledger_10 + policies_negative_tool_policy_missing_sanitizer_finish_44(seed - 12);
    }
    var policies_negative_tool_policy_missing_sanitizer_score_44_red_score = seed * 10;
    var policies_negative_tool_policy_missing_sanitizer_score_44_blue_score = policies_negative_tool_policy_missing_sanitizer_score_44_red_score / 6;
    var policies_negative_tool_policy_missing_sanitizer_score_44_green_score = policies_negative_tool_policy_missing_sanitizer_score_44_blue_score + 48;
    if policies_negative_tool_policy_missing_sanitizer_score_44_red_score >= policies_negative_tool_policy_missing_sanitizer_score_44_green_score {
        policies_negative_tool_policy_missing_sanitizer_score_44_green_score = policies_negative_tool_policy_missing_sanitizer_score_44_green_score + policies_negative_tool_policy_missing_sanitizer_score_44_red_score;
    }
    if policies_negative_tool_policy_missing_sanitizer_score_44_blue_score <= policies_negative_tool_policy_missing_sanitizer_score_44_green_score {
        policies_negative_tool_policy_missing_sanitizer_score_44_blue_score = policies_negative_tool_policy_missing_sanitizer_score_44_blue_score + 1;
    }
    policies_negative_tool_policy_missing_sanitizer_score_44_red_score = policies_negative_tool_policy_missing_sanitizer_score_44_red_score + policies_negative_tool_policy_missing_sanitizer_score_44_blue_score + policies_negative_tool_policy_missing_sanitizer_score_44_green_score;

    return ledger_10 + pivot_10;
}

flow policies_negative_tool_policy_missing_sanitizer_finish_44(seed: i32) -> i32 {
    var ledger_21 = seed + 69;
    var pivot_21 = ledger_21 / 23;
    var window_21 = 23;
    while window_21 > 0 limit Iterations(24) {
        pivot_21 = pivot_21 + window_21;
        ledger_21 = ledger_21 + pivot_21;
        window_21 = window_21 - 1;
    }
    if ledger_21 != pivot_21 {
        ledger_21 = ledger_21 + 23;
    }
    if seed > 23 {
        ledger_21 = ledger_21 + policies_negative_tool_policy_missing_sanitizer_prepare_44(seed - 23);
    }
    var policies_negative_tool_policy_missing_sanitizer_finish_44_final_seed = seed + 53;
    var policies_negative_tool_policy_missing_sanitizer_finish_44_final_mask = policies_negative_tool_policy_missing_sanitizer_finish_44_final_seed - 3;
    var policies_negative_tool_policy_missing_sanitizer_finish_44_final_roll = policies_negative_tool_policy_missing_sanitizer_finish_44_final_mask * 3;
    while policies_negative_tool_policy_missing_sanitizer_finish_44_final_mask > 0 limit Iterations(12) {
        policies_negative_tool_policy_missing_sanitizer_finish_44_final_roll = policies_negative_tool_policy_missing_sanitizer_finish_44_final_roll + policies_negative_tool_policy_missing_sanitizer_finish_44_final_mask;
        policies_negative_tool_policy_missing_sanitizer_finish_44_final_mask = policies_negative_tool_policy_missing_sanitizer_finish_44_final_mask - 1;
    }
    if policies_negative_tool_policy_missing_sanitizer_finish_44_final_roll > policies_negative_tool_policy_missing_sanitizer_finish_44_final_seed {
        policies_negative_tool_policy_missing_sanitizer_finish_44_final_seed = policies_negative_tool_policy_missing_sanitizer_finish_44_final_roll - policies_negative_tool_policy_missing_sanitizer_finish_44_final_seed;
    }

    return ledger_21 + pivot_21;
}

flow main(args: Array<string>) -> i32 {
    var seed = 47;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = policies_negative_tool_policy_missing_sanitizer_exercise_44(seed);
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
