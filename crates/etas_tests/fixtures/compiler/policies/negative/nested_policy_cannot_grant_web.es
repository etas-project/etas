module tests.compiler.policies.negative.nested_policy_cannot_grant_web;

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

spec FixturePolicy: trace = -Web;

spec InnerPolicy: trace = +Web.search;

flow policies_negative_nested_policy_cannot_grant_web_inner_grant_40(seed: i32) -> i32
    ~ InnerPolicy
{
    var local = seed + 40;
    local = local + perform Web.search("inner grant");
    if local > 100 {
        local = local - 10;
    } else {
        local = local + 10;
    }
    local = local + policies_negative_nested_policy_cannot_grant_web_score_40(1);
    var policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_marker = seed + 57;
    var policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_weight = policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_marker * 2;
    var policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_budget = policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_weight - 40;
    if policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_budget > policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_marker {
        policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_marker = policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_marker + policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_budget;
    }
    policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_weight = policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_weight + policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_marker;
    policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_budget = policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_budget + policies_negative_nested_policy_cannot_grant_web_inner_grant_40_inner_weight;

    return local;
}

flow policies_negative_nested_policy_cannot_grant_web_exercise_40(seed: i32) -> i32
    ~ FixturePolicy
{
    var total = policies_negative_nested_policy_cannot_grant_web_prepare_40(seed);
    total = total + policies_negative_nested_policy_cannot_grant_web_route_40(seed + 2);
    if total < 0 {
        total = 0 - total;
    }
    let policies_negative_nested_policy_cannot_grant_web_adjust_40: i32 -> i32 = (value: i32) => value + 40;
    total = policies_negative_nested_policy_cannot_grant_web_adjust_40(total);
    total = total + policies_negative_nested_policy_cannot_grant_web_inner_grant_40(seed);
    total = total + policies_negative_nested_policy_cannot_grant_web_score_40(2);
    total = total + policies_negative_nested_policy_cannot_grant_web_finish_40(1);
    if total > 4000 {
        total = total - 4000;
    } else {
        total = total + 71;
    }
    var policies_negative_nested_policy_cannot_grant_web_exercise_40_lambda_probe = seed + 53;
    var policies_negative_nested_policy_cannot_grant_web_exercise_40_lambda_shadow = policies_negative_nested_policy_cannot_grant_web_exercise_40_lambda_probe * 3;
    var policies_negative_nested_policy_cannot_grant_web_exercise_40_lambda_offset = policies_negative_nested_policy_cannot_grant_web_exercise_40_lambda_shadow - total;
    if policies_negative_nested_policy_cannot_grant_web_exercise_40_lambda_offset > 0 {
        total = total + policies_negative_nested_policy_cannot_grant_web_exercise_40_lambda_offset;
    } else {
        total = total - policies_negative_nested_policy_cannot_grant_web_exercise_40_lambda_offset;
    }
    policies_negative_nested_policy_cannot_grant_web_exercise_40_lambda_probe = policies_negative_nested_policy_cannot_grant_web_exercise_40_lambda_probe + total;
    policies_negative_nested_policy_cannot_grant_web_exercise_40_lambda_shadow = policies_negative_nested_policy_cannot_grant_web_exercise_40_lambda_shadow + policies_negative_nested_policy_cannot_grant_web_exercise_40_lambda_probe;

    return total;
}

flow policies_negative_nested_policy_cannot_grant_web_prepare_40(seed: i32) -> i32 {
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
        ledger_5 = ledger_5 + policies_negative_nested_policy_cannot_grant_web_route_40(seed - 7);
    }
    var policies_negative_nested_policy_cannot_grant_web_prepare_40_alpha_lane = seed + 40;
    var policies_negative_nested_policy_cannot_grant_web_prepare_40_beta_lane = policies_negative_nested_policy_cannot_grant_web_prepare_40_alpha_lane * 2;
    var policies_negative_nested_policy_cannot_grant_web_prepare_40_gamma_lane = policies_negative_nested_policy_cannot_grant_web_prepare_40_beta_lane - seed;
    policies_negative_nested_policy_cannot_grant_web_prepare_40_alpha_lane = policies_negative_nested_policy_cannot_grant_web_prepare_40_alpha_lane + policies_negative_nested_policy_cannot_grant_web_prepare_40_gamma_lane;
    policies_negative_nested_policy_cannot_grant_web_prepare_40_beta_lane = policies_negative_nested_policy_cannot_grant_web_prepare_40_beta_lane + policies_negative_nested_policy_cannot_grant_web_prepare_40_alpha_lane;
    policies_negative_nested_policy_cannot_grant_web_prepare_40_gamma_lane = policies_negative_nested_policy_cannot_grant_web_prepare_40_gamma_lane + policies_negative_nested_policy_cannot_grant_web_prepare_40_beta_lane;
    if policies_negative_nested_policy_cannot_grant_web_prepare_40_gamma_lane > policies_negative_nested_policy_cannot_grant_web_prepare_40_alpha_lane {
        policies_negative_nested_policy_cannot_grant_web_prepare_40_alpha_lane = policies_negative_nested_policy_cannot_grant_web_prepare_40_alpha_lane + 3;
    } else {
        policies_negative_nested_policy_cannot_grant_web_prepare_40_beta_lane = policies_negative_nested_policy_cannot_grant_web_prepare_40_beta_lane + 5;
    }

    return ledger_5 + pivot_5;
}

flow policies_negative_nested_policy_cannot_grant_web_route_40(seed: i32) -> i32 {
    var ledger_10 = seed - 12;
    var pivot_10 = seed + 24;
    var window_10 = 1;
    while window_10 <= 14 limit Iterations(14) {
        ledger_10 = ledger_10 + (pivot_10 % (17));
        pivot_10 = pivot_10 + window_10;
        window_10 = window_10 + 1;
    }
    if ledger_10 < pivot_10 {
        ledger_10 = ledger_10 + pivot_10 - 12;
    }
    if seed > 12 {
        ledger_10 = ledger_10 + policies_negative_nested_policy_cannot_grant_web_score_40(seed - 12);
    }
    var policies_negative_nested_policy_cannot_grant_web_route_40_north_gate = seed + 41;
    var policies_negative_nested_policy_cannot_grant_web_route_40_south_gate = policies_negative_nested_policy_cannot_grant_web_route_40_north_gate % 10;
    var policies_negative_nested_policy_cannot_grant_web_route_40_east_gate = policies_negative_nested_policy_cannot_grant_web_route_40_south_gate + policies_negative_nested_policy_cannot_grant_web_route_40_north_gate;
    while policies_negative_nested_policy_cannot_grant_web_route_40_south_gate < 4 limit Iterations(4) {
        policies_negative_nested_policy_cannot_grant_web_route_40_east_gate = policies_negative_nested_policy_cannot_grant_web_route_40_east_gate + policies_negative_nested_policy_cannot_grant_web_route_40_south_gate;
        policies_negative_nested_policy_cannot_grant_web_route_40_south_gate = policies_negative_nested_policy_cannot_grant_web_route_40_south_gate + 1;
    }
    if policies_negative_nested_policy_cannot_grant_web_route_40_east_gate != policies_negative_nested_policy_cannot_grant_web_route_40_north_gate {
        policies_negative_nested_policy_cannot_grant_web_route_40_north_gate = policies_negative_nested_policy_cannot_grant_web_route_40_north_gate + policies_negative_nested_policy_cannot_grant_web_route_40_east_gate;
    }

    return ledger_10 + pivot_10;
}

flow policies_negative_nested_policy_cannot_grant_web_score_40(seed: i32) -> i32 {
    var ledger_6 = seed * 8;
    var pivot_6 = ledger_6 + 17;
    var window_6 = 0;
    while window_6 < 12 limit Iterations(12) {
        if window_6 % 2 == 0 {
            pivot_6 = pivot_6 + window_6;
        } else {
            ledger_6 = ledger_6 + 8;
        }
        window_6 = window_6 + 1;
    }
    if seed > 8 {
        ledger_6 = ledger_6 + policies_negative_nested_policy_cannot_grant_web_finish_40(seed - 8);
    }
    var policies_negative_nested_policy_cannot_grant_web_score_40_red_score = seed * 6;
    var policies_negative_nested_policy_cannot_grant_web_score_40_blue_score = policies_negative_nested_policy_cannot_grant_web_score_40_red_score / 2;
    var policies_negative_nested_policy_cannot_grant_web_score_40_green_score = policies_negative_nested_policy_cannot_grant_web_score_40_blue_score + 44;
    if policies_negative_nested_policy_cannot_grant_web_score_40_red_score >= policies_negative_nested_policy_cannot_grant_web_score_40_green_score {
        policies_negative_nested_policy_cannot_grant_web_score_40_green_score = policies_negative_nested_policy_cannot_grant_web_score_40_green_score + policies_negative_nested_policy_cannot_grant_web_score_40_red_score;
    }
    if policies_negative_nested_policy_cannot_grant_web_score_40_blue_score <= policies_negative_nested_policy_cannot_grant_web_score_40_green_score {
        policies_negative_nested_policy_cannot_grant_web_score_40_blue_score = policies_negative_nested_policy_cannot_grant_web_score_40_blue_score + 1;
    }
    policies_negative_nested_policy_cannot_grant_web_score_40_red_score = policies_negative_nested_policy_cannot_grant_web_score_40_red_score + policies_negative_nested_policy_cannot_grant_web_score_40_blue_score + policies_negative_nested_policy_cannot_grant_web_score_40_green_score;

    return ledger_6 + pivot_6;
}

flow policies_negative_nested_policy_cannot_grant_web_finish_40(seed: i32) -> i32 {
    var ledger_17 = seed + 57;
    var pivot_17 = ledger_17 / 19;
    var window_17 = 19;
    while window_17 > 0 limit Iterations(20) {
        pivot_17 = pivot_17 + window_17;
        ledger_17 = ledger_17 + pivot_17;
        window_17 = window_17 - 1;
    }
    if ledger_17 != pivot_17 {
        ledger_17 = ledger_17 + 19;
    }
    if seed > 19 {
        ledger_17 = ledger_17 + policies_negative_nested_policy_cannot_grant_web_prepare_40(seed - 19);
    }
    var policies_negative_nested_policy_cannot_grant_web_finish_40_final_seed = seed + 49;
    var policies_negative_nested_policy_cannot_grant_web_finish_40_final_mask = policies_negative_nested_policy_cannot_grant_web_finish_40_final_seed - 5;
    var policies_negative_nested_policy_cannot_grant_web_finish_40_final_roll = policies_negative_nested_policy_cannot_grant_web_finish_40_final_mask * 3;
    while policies_negative_nested_policy_cannot_grant_web_finish_40_final_mask > 0 limit Iterations(12) {
        policies_negative_nested_policy_cannot_grant_web_finish_40_final_roll = policies_negative_nested_policy_cannot_grant_web_finish_40_final_roll + policies_negative_nested_policy_cannot_grant_web_finish_40_final_mask;
        policies_negative_nested_policy_cannot_grant_web_finish_40_final_mask = policies_negative_nested_policy_cannot_grant_web_finish_40_final_mask - 1;
    }
    if policies_negative_nested_policy_cannot_grant_web_finish_40_final_roll > policies_negative_nested_policy_cannot_grant_web_finish_40_final_seed {
        policies_negative_nested_policy_cannot_grant_web_finish_40_final_seed = policies_negative_nested_policy_cannot_grant_web_finish_40_final_roll - policies_negative_nested_policy_cannot_grant_web_finish_40_final_seed;
    }

    return ledger_17 + pivot_17;
}

flow main(args: Array<string>) -> i32 {
    var seed = 43;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = policies_negative_nested_policy_cannot_grant_web_exercise_40(seed);
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
