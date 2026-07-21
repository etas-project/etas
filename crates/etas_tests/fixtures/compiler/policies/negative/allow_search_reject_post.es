module tests.compiler.policies.negative.allow_search_reject_post;

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

spec FixturePolicy: trace = +Web.search;

flow policies_negative_allow_search_reject_post_exercise_31(seed: i32) -> i32
    ~ FixturePolicy
{
    var total = policies_negative_allow_search_reject_post_prepare_31(seed);
    total = total + policies_negative_allow_search_reject_post_route_31(seed + 2);
    if total < 0 {
        total = 0 - total;
    }
    let policies_negative_allow_search_reject_post_adjust_31: i32 -> i32 = (value: i32) => value + 31;
    total = policies_negative_allow_search_reject_post_adjust_31(total);
    perform Web.post("api.example.com", "payload");
    total = total + 5;
    total = total + policies_negative_allow_search_reject_post_score_31(2);
    total = total + policies_negative_allow_search_reject_post_finish_31(1);
    if total > 4000 {
        total = total - 4000;
    } else {
        total = total + 62;
    }
    var policies_negative_allow_search_reject_post_exercise_31_lambda_probe = seed + 44;
    var policies_negative_allow_search_reject_post_exercise_31_lambda_shadow = policies_negative_allow_search_reject_post_exercise_31_lambda_probe * 3;
    var policies_negative_allow_search_reject_post_exercise_31_lambda_offset = policies_negative_allow_search_reject_post_exercise_31_lambda_shadow - total;
    if policies_negative_allow_search_reject_post_exercise_31_lambda_offset > 0 {
        total = total + policies_negative_allow_search_reject_post_exercise_31_lambda_offset;
    } else {
        total = total - policies_negative_allow_search_reject_post_exercise_31_lambda_offset;
    }
    policies_negative_allow_search_reject_post_exercise_31_lambda_probe = policies_negative_allow_search_reject_post_exercise_31_lambda_probe + total;
    policies_negative_allow_search_reject_post_exercise_31_lambda_shadow = policies_negative_allow_search_reject_post_exercise_31_lambda_shadow + policies_negative_allow_search_reject_post_exercise_31_lambda_probe;

    return total;
}

flow policies_negative_allow_search_reject_post_prepare_31(seed: i32) -> i32 {
    var ledger_5 = seed + 21;
    var pivot_5 = ledger_5 / 7;
    var window_5 = 7;
    while window_5 > 0 limit Iterations(8) {
        pivot_5 = pivot_5 + window_5;
        ledger_5 = ledger_5 + pivot_5;
        window_5 = window_5 - 1;
    }
    if ledger_5 != pivot_5 {
        ledger_5 = ledger_5 + 7;
    }
    if seed > 7 {
        ledger_5 = ledger_5 + policies_negative_allow_search_reject_post_route_31(seed - 7);
    }
    var policies_negative_allow_search_reject_post_prepare_31_alpha_lane = seed + 31;
    var policies_negative_allow_search_reject_post_prepare_31_beta_lane = policies_negative_allow_search_reject_post_prepare_31_alpha_lane * 2;
    var policies_negative_allow_search_reject_post_prepare_31_gamma_lane = policies_negative_allow_search_reject_post_prepare_31_beta_lane - seed;
    policies_negative_allow_search_reject_post_prepare_31_alpha_lane = policies_negative_allow_search_reject_post_prepare_31_alpha_lane + policies_negative_allow_search_reject_post_prepare_31_gamma_lane;
    policies_negative_allow_search_reject_post_prepare_31_beta_lane = policies_negative_allow_search_reject_post_prepare_31_beta_lane + policies_negative_allow_search_reject_post_prepare_31_alpha_lane;
    policies_negative_allow_search_reject_post_prepare_31_gamma_lane = policies_negative_allow_search_reject_post_prepare_31_gamma_lane + policies_negative_allow_search_reject_post_prepare_31_beta_lane;
    if policies_negative_allow_search_reject_post_prepare_31_gamma_lane > policies_negative_allow_search_reject_post_prepare_31_alpha_lane {
        policies_negative_allow_search_reject_post_prepare_31_alpha_lane = policies_negative_allow_search_reject_post_prepare_31_alpha_lane + 3;
    } else {
        policies_negative_allow_search_reject_post_prepare_31_beta_lane = policies_negative_allow_search_reject_post_prepare_31_beta_lane + 5;
    }

    return ledger_5 + pivot_5;
}

flow policies_negative_allow_search_reject_post_route_31(seed: i32) -> i32 {
    var ledger_12 = seed + 14;
    var pivot_12 = ledger_12 * 15;
    var window_12 = 0;
    while window_12 < 17 limit Iterations(17) {
        pivot_12 = pivot_12 + window_12 + 14;
        window_12 = window_12 + 1;
    }
    if pivot_12 > 98 {
        ledger_12 = ledger_12 + pivot_12;
    } else {
        ledger_12 = ledger_12 - 14;
    }
    if seed > 14 {
        ledger_12 = ledger_12 + policies_negative_allow_search_reject_post_score_31(seed - 14);
    }
    var policies_negative_allow_search_reject_post_route_31_north_gate = seed + 32;
    var policies_negative_allow_search_reject_post_route_31_south_gate = policies_negative_allow_search_reject_post_route_31_north_gate % 8;
    var policies_negative_allow_search_reject_post_route_31_east_gate = policies_negative_allow_search_reject_post_route_31_south_gate + policies_negative_allow_search_reject_post_route_31_north_gate;
    while policies_negative_allow_search_reject_post_route_31_south_gate < 4 limit Iterations(4) {
        policies_negative_allow_search_reject_post_route_31_east_gate = policies_negative_allow_search_reject_post_route_31_east_gate + policies_negative_allow_search_reject_post_route_31_south_gate;
        policies_negative_allow_search_reject_post_route_31_south_gate = policies_negative_allow_search_reject_post_route_31_south_gate + 1;
    }
    if policies_negative_allow_search_reject_post_route_31_east_gate != policies_negative_allow_search_reject_post_route_31_north_gate {
        policies_negative_allow_search_reject_post_route_31_north_gate = policies_negative_allow_search_reject_post_route_31_north_gate + policies_negative_allow_search_reject_post_route_31_east_gate;
    }

    return ledger_12 + pivot_12;
}

flow policies_negative_allow_search_reject_post_score_31(seed: i32) -> i32 {
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
        ledger_10 = ledger_10 + policies_negative_allow_search_reject_post_finish_31(seed - 12);
    }
    var policies_negative_allow_search_reject_post_score_31_red_score = seed * 6;
    var policies_negative_allow_search_reject_post_score_31_blue_score = policies_negative_allow_search_reject_post_score_31_red_score / 3;
    var policies_negative_allow_search_reject_post_score_31_green_score = policies_negative_allow_search_reject_post_score_31_blue_score + 35;
    if policies_negative_allow_search_reject_post_score_31_red_score >= policies_negative_allow_search_reject_post_score_31_green_score {
        policies_negative_allow_search_reject_post_score_31_green_score = policies_negative_allow_search_reject_post_score_31_green_score + policies_negative_allow_search_reject_post_score_31_red_score;
    }
    if policies_negative_allow_search_reject_post_score_31_blue_score <= policies_negative_allow_search_reject_post_score_31_green_score {
        policies_negative_allow_search_reject_post_score_31_blue_score = policies_negative_allow_search_reject_post_score_31_blue_score + 1;
    }
    policies_negative_allow_search_reject_post_score_31_red_score = policies_negative_allow_search_reject_post_score_31_red_score + policies_negative_allow_search_reject_post_score_31_blue_score + policies_negative_allow_search_reject_post_score_31_green_score;

    return ledger_10 + pivot_10;
}

flow policies_negative_allow_search_reject_post_finish_31(seed: i32) -> i32 {
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
        ledger_8 = ledger_8 + policies_negative_allow_search_reject_post_prepare_31(seed - 10);
    }
    var policies_negative_allow_search_reject_post_finish_31_final_seed = seed + 40;
    var policies_negative_allow_search_reject_post_finish_31_final_mask = policies_negative_allow_search_reject_post_finish_31_final_seed - 2;
    var policies_negative_allow_search_reject_post_finish_31_final_roll = policies_negative_allow_search_reject_post_finish_31_final_mask * 6;
    while policies_negative_allow_search_reject_post_finish_31_final_mask > 0 limit Iterations(12) {
        policies_negative_allow_search_reject_post_finish_31_final_roll = policies_negative_allow_search_reject_post_finish_31_final_roll + policies_negative_allow_search_reject_post_finish_31_final_mask;
        policies_negative_allow_search_reject_post_finish_31_final_mask = policies_negative_allow_search_reject_post_finish_31_final_mask - 1;
    }
    if policies_negative_allow_search_reject_post_finish_31_final_roll > policies_negative_allow_search_reject_post_finish_31_final_seed {
        policies_negative_allow_search_reject_post_finish_31_final_seed = policies_negative_allow_search_reject_post_finish_31_final_roll - policies_negative_allow_search_reject_post_finish_31_final_seed;
    }

    return ledger_8 + pivot_8;
}

flow main(args: Array<string>) -> i32 {
    var seed = 34;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = policies_negative_allow_search_reject_post_exercise_31(seed);
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
