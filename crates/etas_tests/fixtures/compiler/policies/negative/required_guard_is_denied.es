module tests.compiler.policies.negative.required_guard_is_denied;

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

spec FixturePolicy: trace = +Email.send & -Approval.request & (Approval.request >> Email.send);

flow policies_negative_required_guard_is_denied_exercise_43(seed: i32) -> i32
    ~ FixturePolicy
{
    var total = policies_negative_required_guard_is_denied_prepare_43(seed);
    total = total + policies_negative_required_guard_is_denied_route_43(seed + 2);
    if total < 0 {
        total = 0 - total;
    }
    let policies_negative_required_guard_is_denied_adjust_43: i32 -> i32 = (value: i32) => value + 43;
    total = policies_negative_required_guard_is_denied_adjust_43(total);
    perform Approval.request("send");
    perform Email.send("WorkAccount", "body");
    total = total + 8;
    total = total + policies_negative_required_guard_is_denied_score_43(2);
    total = total + policies_negative_required_guard_is_denied_finish_43(1);
    if total > 4000 {
        total = total - 4000;
    } else {
        total = total + 74;
    }
    var policies_negative_required_guard_is_denied_exercise_43_lambda_probe = seed + 56;
    var policies_negative_required_guard_is_denied_exercise_43_lambda_shadow = policies_negative_required_guard_is_denied_exercise_43_lambda_probe * 3;
    var policies_negative_required_guard_is_denied_exercise_43_lambda_offset = policies_negative_required_guard_is_denied_exercise_43_lambda_shadow - total;
    if policies_negative_required_guard_is_denied_exercise_43_lambda_offset > 0 {
        total = total + policies_negative_required_guard_is_denied_exercise_43_lambda_offset;
    } else {
        total = total - policies_negative_required_guard_is_denied_exercise_43_lambda_offset;
    }
    policies_negative_required_guard_is_denied_exercise_43_lambda_probe = policies_negative_required_guard_is_denied_exercise_43_lambda_probe + total;
    policies_negative_required_guard_is_denied_exercise_43_lambda_shadow = policies_negative_required_guard_is_denied_exercise_43_lambda_shadow + policies_negative_required_guard_is_denied_exercise_43_lambda_probe;

    return total;
}

flow policies_negative_required_guard_is_denied_prepare_43(seed: i32) -> i32 {
    var ledger_8 = seed + 30;
    var pivot_8 = ledger_8 / 10;
    var window_8 = 10;
    while window_8 > 0 limit Iterations(11) {
        pivot_8 = pivot_8 + window_8;
        ledger_8 = ledger_8 + pivot_8;
        window_8 = window_8 - 1;
    }
    if ledger_8 != pivot_8 {
        ledger_8 = ledger_8 + 10;
    }
    if seed > 10 {
        ledger_8 = ledger_8 + policies_negative_required_guard_is_denied_route_43(seed - 10);
    }
    var policies_negative_required_guard_is_denied_prepare_43_alpha_lane = seed + 43;
    var policies_negative_required_guard_is_denied_prepare_43_beta_lane = policies_negative_required_guard_is_denied_prepare_43_alpha_lane * 2;
    var policies_negative_required_guard_is_denied_prepare_43_gamma_lane = policies_negative_required_guard_is_denied_prepare_43_beta_lane - seed;
    policies_negative_required_guard_is_denied_prepare_43_alpha_lane = policies_negative_required_guard_is_denied_prepare_43_alpha_lane + policies_negative_required_guard_is_denied_prepare_43_gamma_lane;
    policies_negative_required_guard_is_denied_prepare_43_beta_lane = policies_negative_required_guard_is_denied_prepare_43_beta_lane + policies_negative_required_guard_is_denied_prepare_43_alpha_lane;
    policies_negative_required_guard_is_denied_prepare_43_gamma_lane = policies_negative_required_guard_is_denied_prepare_43_gamma_lane + policies_negative_required_guard_is_denied_prepare_43_beta_lane;
    if policies_negative_required_guard_is_denied_prepare_43_gamma_lane > policies_negative_required_guard_is_denied_prepare_43_alpha_lane {
        policies_negative_required_guard_is_denied_prepare_43_alpha_lane = policies_negative_required_guard_is_denied_prepare_43_alpha_lane + 3;
    } else {
        policies_negative_required_guard_is_denied_prepare_43_beta_lane = policies_negative_required_guard_is_denied_prepare_43_beta_lane + 5;
    }

    return ledger_8 + pivot_8;
}

flow policies_negative_required_guard_is_denied_route_43(seed: i32) -> i32 {
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
        ledger_13 = ledger_13 + policies_negative_required_guard_is_denied_score_43(seed - 15);
    }
    var policies_negative_required_guard_is_denied_route_43_north_gate = seed + 44;
    var policies_negative_required_guard_is_denied_route_43_south_gate = policies_negative_required_guard_is_denied_route_43_north_gate % 6;
    var policies_negative_required_guard_is_denied_route_43_east_gate = policies_negative_required_guard_is_denied_route_43_south_gate + policies_negative_required_guard_is_denied_route_43_north_gate;
    while policies_negative_required_guard_is_denied_route_43_south_gate < 4 limit Iterations(4) {
        policies_negative_required_guard_is_denied_route_43_east_gate = policies_negative_required_guard_is_denied_route_43_east_gate + policies_negative_required_guard_is_denied_route_43_south_gate;
        policies_negative_required_guard_is_denied_route_43_south_gate = policies_negative_required_guard_is_denied_route_43_south_gate + 1;
    }
    if policies_negative_required_guard_is_denied_route_43_east_gate != policies_negative_required_guard_is_denied_route_43_north_gate {
        policies_negative_required_guard_is_denied_route_43_north_gate = policies_negative_required_guard_is_denied_route_43_north_gate + policies_negative_required_guard_is_denied_route_43_east_gate;
    }

    return ledger_13 + pivot_13;
}

flow policies_negative_required_guard_is_denied_score_43(seed: i32) -> i32 {
    var ledger_9 = seed - 11;
    var pivot_9 = seed + 22;
    var window_9 = 1;
    while window_9 <= 13 limit Iterations(13) {
        ledger_9 = ledger_9 + (pivot_9 % (16));
        pivot_9 = pivot_9 + window_9;
        window_9 = window_9 + 1;
    }
    if ledger_9 < pivot_9 {
        ledger_9 = ledger_9 + pivot_9 - 11;
    }
    if seed > 11 {
        ledger_9 = ledger_9 + policies_negative_required_guard_is_denied_finish_43(seed - 11);
    }
    var policies_negative_required_guard_is_denied_score_43_red_score = seed * 9;
    var policies_negative_required_guard_is_denied_score_43_blue_score = policies_negative_required_guard_is_denied_score_43_red_score / 5;
    var policies_negative_required_guard_is_denied_score_43_green_score = policies_negative_required_guard_is_denied_score_43_blue_score + 47;
    if policies_negative_required_guard_is_denied_score_43_red_score >= policies_negative_required_guard_is_denied_score_43_green_score {
        policies_negative_required_guard_is_denied_score_43_green_score = policies_negative_required_guard_is_denied_score_43_green_score + policies_negative_required_guard_is_denied_score_43_red_score;
    }
    if policies_negative_required_guard_is_denied_score_43_blue_score <= policies_negative_required_guard_is_denied_score_43_green_score {
        policies_negative_required_guard_is_denied_score_43_blue_score = policies_negative_required_guard_is_denied_score_43_blue_score + 1;
    }
    policies_negative_required_guard_is_denied_score_43_red_score = policies_negative_required_guard_is_denied_score_43_red_score + policies_negative_required_guard_is_denied_score_43_blue_score + policies_negative_required_guard_is_denied_score_43_green_score;

    return ledger_9 + pivot_9;
}

flow policies_negative_required_guard_is_denied_finish_43(seed: i32) -> i32 {
    var ledger_20 = seed * 22;
    var pivot_20 = ledger_20 + 31;
    var window_20 = 0;
    while window_20 < 26 limit Iterations(26) {
        if window_20 % 2 == 0 {
            pivot_20 = pivot_20 + window_20;
        } else {
            ledger_20 = ledger_20 + 22;
        }
        window_20 = window_20 + 1;
    }
    if seed > 22 {
        ledger_20 = ledger_20 + policies_negative_required_guard_is_denied_prepare_43(seed - 22);
    }
    var policies_negative_required_guard_is_denied_finish_43_final_seed = seed + 52;
    var policies_negative_required_guard_is_denied_finish_43_final_mask = policies_negative_required_guard_is_denied_finish_43_final_seed - 2;
    var policies_negative_required_guard_is_denied_finish_43_final_roll = policies_negative_required_guard_is_denied_finish_43_final_mask * 6;
    while policies_negative_required_guard_is_denied_finish_43_final_mask > 0 limit Iterations(12) {
        policies_negative_required_guard_is_denied_finish_43_final_roll = policies_negative_required_guard_is_denied_finish_43_final_roll + policies_negative_required_guard_is_denied_finish_43_final_mask;
        policies_negative_required_guard_is_denied_finish_43_final_mask = policies_negative_required_guard_is_denied_finish_43_final_mask - 1;
    }
    if policies_negative_required_guard_is_denied_finish_43_final_roll > policies_negative_required_guard_is_denied_finish_43_final_seed {
        policies_negative_required_guard_is_denied_finish_43_final_seed = policies_negative_required_guard_is_denied_finish_43_final_roll - policies_negative_required_guard_is_denied_finish_43_final_seed;
    }

    return ledger_20 + pivot_20;
}

flow main(args: Array<string>) -> i32 {
    var seed = 46;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = policies_negative_required_guard_is_denied_exercise_43(seed);
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
