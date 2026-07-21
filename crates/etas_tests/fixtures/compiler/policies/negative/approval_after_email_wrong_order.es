module tests.compiler.policies.negative.approval_after_email_wrong_order;

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

spec FixturePolicy: trace = +Email.send & (Approval.request >> Email.send);

flow policies_negative_approval_after_email_wrong_order_exercise_32(seed: i32) -> i32
    ~ FixturePolicy
{
    var total = policies_negative_approval_after_email_wrong_order_prepare_32(seed);
    total = total + policies_negative_approval_after_email_wrong_order_route_32(seed + 2);
    if total < 0 {
        total = 0 - total;
    }
    let policies_negative_approval_after_email_wrong_order_adjust_32: i32 -> i32 = (value: i32) => value + 32;
    total = policies_negative_approval_after_email_wrong_order_adjust_32(total);
    perform Email.send("WorkAccount", "body");
    perform Approval.request("late approval");
    total = total + 6;
    total = total + policies_negative_approval_after_email_wrong_order_score_32(2);
    total = total + policies_negative_approval_after_email_wrong_order_finish_32(1);
    if total > 4000 {
        total = total - 4000;
    } else {
        total = total + 63;
    }
    var policies_negative_approval_after_email_wrong_order_exercise_32_lambda_probe = seed + 45;
    var policies_negative_approval_after_email_wrong_order_exercise_32_lambda_shadow = policies_negative_approval_after_email_wrong_order_exercise_32_lambda_probe * 3;
    var policies_negative_approval_after_email_wrong_order_exercise_32_lambda_offset = policies_negative_approval_after_email_wrong_order_exercise_32_lambda_shadow - total;
    if policies_negative_approval_after_email_wrong_order_exercise_32_lambda_offset > 0 {
        total = total + policies_negative_approval_after_email_wrong_order_exercise_32_lambda_offset;
    } else {
        total = total - policies_negative_approval_after_email_wrong_order_exercise_32_lambda_offset;
    }
    policies_negative_approval_after_email_wrong_order_exercise_32_lambda_probe = policies_negative_approval_after_email_wrong_order_exercise_32_lambda_probe + total;
    policies_negative_approval_after_email_wrong_order_exercise_32_lambda_shadow = policies_negative_approval_after_email_wrong_order_exercise_32_lambda_shadow + policies_negative_approval_after_email_wrong_order_exercise_32_lambda_probe;

    return total;
}

flow policies_negative_approval_after_email_wrong_order_prepare_32(seed: i32) -> i32 {
    var ledger_6 = seed + 8;
    var pivot_6 = ledger_6 * 9;
    var window_6 = 0;
    while window_6 < 11 limit Iterations(11) {
        pivot_6 = pivot_6 + window_6 + 8;
        window_6 = window_6 + 1;
    }
    if pivot_6 > 56 {
        ledger_6 = ledger_6 + pivot_6;
    } else {
        ledger_6 = ledger_6 - 8;
    }
    if seed > 8 {
        ledger_6 = ledger_6 + policies_negative_approval_after_email_wrong_order_route_32(seed - 8);
    }
    var policies_negative_approval_after_email_wrong_order_prepare_32_alpha_lane = seed + 32;
    var policies_negative_approval_after_email_wrong_order_prepare_32_beta_lane = policies_negative_approval_after_email_wrong_order_prepare_32_alpha_lane * 2;
    var policies_negative_approval_after_email_wrong_order_prepare_32_gamma_lane = policies_negative_approval_after_email_wrong_order_prepare_32_beta_lane - seed;
    policies_negative_approval_after_email_wrong_order_prepare_32_alpha_lane = policies_negative_approval_after_email_wrong_order_prepare_32_alpha_lane + policies_negative_approval_after_email_wrong_order_prepare_32_gamma_lane;
    policies_negative_approval_after_email_wrong_order_prepare_32_beta_lane = policies_negative_approval_after_email_wrong_order_prepare_32_beta_lane + policies_negative_approval_after_email_wrong_order_prepare_32_alpha_lane;
    policies_negative_approval_after_email_wrong_order_prepare_32_gamma_lane = policies_negative_approval_after_email_wrong_order_prepare_32_gamma_lane + policies_negative_approval_after_email_wrong_order_prepare_32_beta_lane;
    if policies_negative_approval_after_email_wrong_order_prepare_32_gamma_lane > policies_negative_approval_after_email_wrong_order_prepare_32_alpha_lane {
        policies_negative_approval_after_email_wrong_order_prepare_32_alpha_lane = policies_negative_approval_after_email_wrong_order_prepare_32_alpha_lane + 3;
    } else {
        policies_negative_approval_after_email_wrong_order_prepare_32_beta_lane = policies_negative_approval_after_email_wrong_order_prepare_32_beta_lane + 5;
    }

    return ledger_6 + pivot_6;
}

flow policies_negative_approval_after_email_wrong_order_route_32(seed: i32) -> i32 {
    var ledger_13 = seed - 15;
    var pivot_13 = seed + 30;
    var window_13 = 1;
    while window_13 <= 17 limit Iterations(17) {
        ledger_13 = ledger_13 + (pivot_13 % (20));
        pivot_13 = pivot_13 + window_13;
        window_13 = window_13 + 1;
    }
    if ledger_13 < pivot_13 {
        ledger_13 = ledger_13 + pivot_13 - 15;
    }
    if seed > 15 {
        ledger_13 = ledger_13 + policies_negative_approval_after_email_wrong_order_score_32(seed - 15);
    }
    var policies_negative_approval_after_email_wrong_order_route_32_north_gate = seed + 33;
    var policies_negative_approval_after_email_wrong_order_route_32_south_gate = policies_negative_approval_after_email_wrong_order_route_32_north_gate % 9;
    var policies_negative_approval_after_email_wrong_order_route_32_east_gate = policies_negative_approval_after_email_wrong_order_route_32_south_gate + policies_negative_approval_after_email_wrong_order_route_32_north_gate;
    while policies_negative_approval_after_email_wrong_order_route_32_south_gate < 4 limit Iterations(4) {
        policies_negative_approval_after_email_wrong_order_route_32_east_gate = policies_negative_approval_after_email_wrong_order_route_32_east_gate + policies_negative_approval_after_email_wrong_order_route_32_south_gate;
        policies_negative_approval_after_email_wrong_order_route_32_south_gate = policies_negative_approval_after_email_wrong_order_route_32_south_gate + 1;
    }
    if policies_negative_approval_after_email_wrong_order_route_32_east_gate != policies_negative_approval_after_email_wrong_order_route_32_north_gate {
        policies_negative_approval_after_email_wrong_order_route_32_north_gate = policies_negative_approval_after_email_wrong_order_route_32_north_gate + policies_negative_approval_after_email_wrong_order_route_32_east_gate;
    }

    return ledger_13 + pivot_13;
}

flow policies_negative_approval_after_email_wrong_order_score_32(seed: i32) -> i32 {
    var ledger_11 = seed * 13;
    var pivot_11 = ledger_11 + 22;
    var window_11 = 0;
    while window_11 < 17 limit Iterations(17) {
        if window_11 % 2 == 0 {
            pivot_11 = pivot_11 + window_11;
        } else {
            ledger_11 = ledger_11 + 13;
        }
        window_11 = window_11 + 1;
    }
    if seed > 13 {
        ledger_11 = ledger_11 + policies_negative_approval_after_email_wrong_order_finish_32(seed - 13);
    }
    var policies_negative_approval_after_email_wrong_order_score_32_red_score = seed * 7;
    var policies_negative_approval_after_email_wrong_order_score_32_blue_score = policies_negative_approval_after_email_wrong_order_score_32_red_score / 4;
    var policies_negative_approval_after_email_wrong_order_score_32_green_score = policies_negative_approval_after_email_wrong_order_score_32_blue_score + 36;
    if policies_negative_approval_after_email_wrong_order_score_32_red_score >= policies_negative_approval_after_email_wrong_order_score_32_green_score {
        policies_negative_approval_after_email_wrong_order_score_32_green_score = policies_negative_approval_after_email_wrong_order_score_32_green_score + policies_negative_approval_after_email_wrong_order_score_32_red_score;
    }
    if policies_negative_approval_after_email_wrong_order_score_32_blue_score <= policies_negative_approval_after_email_wrong_order_score_32_green_score {
        policies_negative_approval_after_email_wrong_order_score_32_blue_score = policies_negative_approval_after_email_wrong_order_score_32_blue_score + 1;
    }
    policies_negative_approval_after_email_wrong_order_score_32_red_score = policies_negative_approval_after_email_wrong_order_score_32_red_score + policies_negative_approval_after_email_wrong_order_score_32_blue_score + policies_negative_approval_after_email_wrong_order_score_32_green_score;

    return ledger_11 + pivot_11;
}

flow policies_negative_approval_after_email_wrong_order_finish_32(seed: i32) -> i32 {
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
        ledger_9 = ledger_9 + policies_negative_approval_after_email_wrong_order_prepare_32(seed - 11);
    }
    var policies_negative_approval_after_email_wrong_order_finish_32_final_seed = seed + 41;
    var policies_negative_approval_after_email_wrong_order_finish_32_final_mask = policies_negative_approval_after_email_wrong_order_finish_32_final_seed - 3;
    var policies_negative_approval_after_email_wrong_order_finish_32_final_roll = policies_negative_approval_after_email_wrong_order_finish_32_final_mask * 3;
    while policies_negative_approval_after_email_wrong_order_finish_32_final_mask > 0 limit Iterations(12) {
        policies_negative_approval_after_email_wrong_order_finish_32_final_roll = policies_negative_approval_after_email_wrong_order_finish_32_final_roll + policies_negative_approval_after_email_wrong_order_finish_32_final_mask;
        policies_negative_approval_after_email_wrong_order_finish_32_final_mask = policies_negative_approval_after_email_wrong_order_finish_32_final_mask - 1;
    }
    if policies_negative_approval_after_email_wrong_order_finish_32_final_roll > policies_negative_approval_after_email_wrong_order_finish_32_final_seed {
        policies_negative_approval_after_email_wrong_order_finish_32_final_seed = policies_negative_approval_after_email_wrong_order_finish_32_final_roll - policies_negative_approval_after_email_wrong_order_finish_32_final_seed;
    }

    return ledger_9 + pivot_9;
}

flow main(args: Array<string>) -> i32 {
    var seed = 35;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = policies_negative_approval_after_email_wrong_order_exercise_32(seed);
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
