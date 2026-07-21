module tests.compiler.policies.negative.deny_web_search;

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

flow policies_negative_deny_web_search_exercise_36(seed: i32) -> i32
    ~ FixturePolicy
{
    var total = policies_negative_deny_web_search_prepare_36(seed);
    total = total + policies_negative_deny_web_search_route_36(seed + 2);
    if total < 0 {
        total = 0 - total;
    }
    let policies_negative_deny_web_search_adjust_36: i32 -> i32 = (value: i32) => value + 36;
    total = policies_negative_deny_web_search_adjust_36(total);
    total = total + perform Web.search("blocked");
    total = total + policies_negative_deny_web_search_score_36(2);
    total = total + policies_negative_deny_web_search_finish_36(1);
    if total > 4000 {
        total = total - 4000;
    } else {
        total = total + 67;
    }
    var policies_negative_deny_web_search_exercise_36_lambda_probe = seed + 49;
    var policies_negative_deny_web_search_exercise_36_lambda_shadow = policies_negative_deny_web_search_exercise_36_lambda_probe * 3;
    var policies_negative_deny_web_search_exercise_36_lambda_offset = policies_negative_deny_web_search_exercise_36_lambda_shadow - total;
    if policies_negative_deny_web_search_exercise_36_lambda_offset > 0 {
        total = total + policies_negative_deny_web_search_exercise_36_lambda_offset;
    } else {
        total = total - policies_negative_deny_web_search_exercise_36_lambda_offset;
    }
    policies_negative_deny_web_search_exercise_36_lambda_probe = policies_negative_deny_web_search_exercise_36_lambda_probe + total;
    policies_negative_deny_web_search_exercise_36_lambda_shadow = policies_negative_deny_web_search_exercise_36_lambda_shadow + policies_negative_deny_web_search_exercise_36_lambda_probe;

    return total;
}

flow policies_negative_deny_web_search_prepare_36(seed: i32) -> i32 {
    var ledger_1 = seed + 3;
    var pivot_1 = ledger_1 * 4;
    var window_1 = 0;
    while window_1 < 6 limit Iterations(6) {
        pivot_1 = pivot_1 + window_1 + 3;
        window_1 = window_1 + 1;
    }
    if pivot_1 > 21 {
        ledger_1 = ledger_1 + pivot_1;
    } else {
        ledger_1 = ledger_1 - 3;
    }
    if seed > 3 {
        ledger_1 = ledger_1 + policies_negative_deny_web_search_route_36(seed - 3);
    }
    var policies_negative_deny_web_search_prepare_36_alpha_lane = seed + 36;
    var policies_negative_deny_web_search_prepare_36_beta_lane = policies_negative_deny_web_search_prepare_36_alpha_lane * 2;
    var policies_negative_deny_web_search_prepare_36_gamma_lane = policies_negative_deny_web_search_prepare_36_beta_lane - seed;
    policies_negative_deny_web_search_prepare_36_alpha_lane = policies_negative_deny_web_search_prepare_36_alpha_lane + policies_negative_deny_web_search_prepare_36_gamma_lane;
    policies_negative_deny_web_search_prepare_36_beta_lane = policies_negative_deny_web_search_prepare_36_beta_lane + policies_negative_deny_web_search_prepare_36_alpha_lane;
    policies_negative_deny_web_search_prepare_36_gamma_lane = policies_negative_deny_web_search_prepare_36_gamma_lane + policies_negative_deny_web_search_prepare_36_beta_lane;
    if policies_negative_deny_web_search_prepare_36_gamma_lane > policies_negative_deny_web_search_prepare_36_alpha_lane {
        policies_negative_deny_web_search_prepare_36_alpha_lane = policies_negative_deny_web_search_prepare_36_alpha_lane + 3;
    } else {
        policies_negative_deny_web_search_prepare_36_beta_lane = policies_negative_deny_web_search_prepare_36_beta_lane + 5;
    }

    return ledger_1 + pivot_1;
}

flow policies_negative_deny_web_search_route_36(seed: i32) -> i32 {
    var ledger_6 = seed - 8;
    var pivot_6 = seed + 16;
    var window_6 = 1;
    while window_6 <= 10 limit Iterations(10) {
        ledger_6 = ledger_6 + (pivot_6 % (13));
        pivot_6 = pivot_6 + window_6;
        window_6 = window_6 + 1;
    }
    if ledger_6 < pivot_6 {
        ledger_6 = ledger_6 + pivot_6 - 8;
    }
    if seed > 8 {
        ledger_6 = ledger_6 + policies_negative_deny_web_search_score_36(seed - 8);
    }
    var policies_negative_deny_web_search_route_36_north_gate = seed + 37;
    var policies_negative_deny_web_search_route_36_south_gate = policies_negative_deny_web_search_route_36_north_gate % 6;
    var policies_negative_deny_web_search_route_36_east_gate = policies_negative_deny_web_search_route_36_south_gate + policies_negative_deny_web_search_route_36_north_gate;
    while policies_negative_deny_web_search_route_36_south_gate < 4 limit Iterations(4) {
        policies_negative_deny_web_search_route_36_east_gate = policies_negative_deny_web_search_route_36_east_gate + policies_negative_deny_web_search_route_36_south_gate;
        policies_negative_deny_web_search_route_36_south_gate = policies_negative_deny_web_search_route_36_south_gate + 1;
    }
    if policies_negative_deny_web_search_route_36_east_gate != policies_negative_deny_web_search_route_36_north_gate {
        policies_negative_deny_web_search_route_36_north_gate = policies_negative_deny_web_search_route_36_north_gate + policies_negative_deny_web_search_route_36_east_gate;
    }

    return ledger_6 + pivot_6;
}

flow policies_negative_deny_web_search_score_36(seed: i32) -> i32 {
    var ledger_15 = seed * 17;
    var pivot_15 = ledger_15 + 26;
    var window_15 = 0;
    while window_15 < 21 limit Iterations(21) {
        if window_15 % 2 == 0 {
            pivot_15 = pivot_15 + window_15;
        } else {
            ledger_15 = ledger_15 + 17;
        }
        window_15 = window_15 + 1;
    }
    if seed > 17 {
        ledger_15 = ledger_15 + policies_negative_deny_web_search_finish_36(seed - 17);
    }
    var policies_negative_deny_web_search_score_36_red_score = seed * 2;
    var policies_negative_deny_web_search_score_36_blue_score = policies_negative_deny_web_search_score_36_red_score / 3;
    var policies_negative_deny_web_search_score_36_green_score = policies_negative_deny_web_search_score_36_blue_score + 40;
    if policies_negative_deny_web_search_score_36_red_score >= policies_negative_deny_web_search_score_36_green_score {
        policies_negative_deny_web_search_score_36_green_score = policies_negative_deny_web_search_score_36_green_score + policies_negative_deny_web_search_score_36_red_score;
    }
    if policies_negative_deny_web_search_score_36_blue_score <= policies_negative_deny_web_search_score_36_green_score {
        policies_negative_deny_web_search_score_36_blue_score = policies_negative_deny_web_search_score_36_blue_score + 1;
    }
    policies_negative_deny_web_search_score_36_red_score = policies_negative_deny_web_search_score_36_red_score + policies_negative_deny_web_search_score_36_blue_score + policies_negative_deny_web_search_score_36_green_score;

    return ledger_15 + pivot_15;
}

flow policies_negative_deny_web_search_finish_36(seed: i32) -> i32 {
    var ledger_13 = seed + 45;
    var pivot_13 = ledger_13 / 15;
    var window_13 = 15;
    while window_13 > 0 limit Iterations(16) {
        pivot_13 = pivot_13 + window_13;
        ledger_13 = ledger_13 + pivot_13;
        window_13 = window_13 - 1;
    }
    if ledger_13 != pivot_13 {
        ledger_13 = ledger_13 + 15;
    }
    if seed > 15 {
        ledger_13 = ledger_13 + policies_negative_deny_web_search_prepare_36(seed - 15);
    }
    var policies_negative_deny_web_search_finish_36_final_seed = seed + 45;
    var policies_negative_deny_web_search_finish_36_final_mask = policies_negative_deny_web_search_finish_36_final_seed - 1;
    var policies_negative_deny_web_search_finish_36_final_roll = policies_negative_deny_web_search_finish_36_final_mask * 3;
    while policies_negative_deny_web_search_finish_36_final_mask > 0 limit Iterations(12) {
        policies_negative_deny_web_search_finish_36_final_roll = policies_negative_deny_web_search_finish_36_final_roll + policies_negative_deny_web_search_finish_36_final_mask;
        policies_negative_deny_web_search_finish_36_final_mask = policies_negative_deny_web_search_finish_36_final_mask - 1;
    }
    if policies_negative_deny_web_search_finish_36_final_roll > policies_negative_deny_web_search_finish_36_final_seed {
        policies_negative_deny_web_search_finish_36_final_seed = policies_negative_deny_web_search_finish_36_final_roll - policies_negative_deny_web_search_finish_36_final_seed;
    }

    return ledger_13 + pivot_13;
}

flow main(args: Array<string>) -> i32 {
    var seed = 39;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = policies_negative_deny_web_search_exercise_36(seed);
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
