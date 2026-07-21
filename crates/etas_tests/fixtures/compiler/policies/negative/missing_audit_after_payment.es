module tests.compiler.policies.negative.missing_audit_after_payment;

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

spec FixturePolicy: trace = +Payment.charge & (Audit.write << Payment.charge);

flow policies_negative_missing_audit_after_payment_exercise_39(seed: i32) -> i32
    ~ FixturePolicy
{
    var total = policies_negative_missing_audit_after_payment_prepare_39(seed);
    total = total + policies_negative_missing_audit_after_payment_route_39(seed + 2);
    if total < 0 {
        total = 0 - total;
    }
    let policies_negative_missing_audit_after_payment_adjust_39: i32 -> i32 = (value: i32) => value + 39;
    total = policies_negative_missing_audit_after_payment_adjust_39(total);
    let charge = perform Payment.charge("BillingAccount", 999);
    total = total + charge.code;
    total = total + policies_negative_missing_audit_after_payment_score_39(2);
    total = total + policies_negative_missing_audit_after_payment_finish_39(1);
    if total > 4000 {
        total = total - 4000;
    } else {
        total = total + 70;
    }
    var policies_negative_missing_audit_after_payment_exercise_39_lambda_probe = seed + 52;
    var policies_negative_missing_audit_after_payment_exercise_39_lambda_shadow = policies_negative_missing_audit_after_payment_exercise_39_lambda_probe * 3;
    var policies_negative_missing_audit_after_payment_exercise_39_lambda_offset = policies_negative_missing_audit_after_payment_exercise_39_lambda_shadow - total;
    if policies_negative_missing_audit_after_payment_exercise_39_lambda_offset > 0 {
        total = total + policies_negative_missing_audit_after_payment_exercise_39_lambda_offset;
    } else {
        total = total - policies_negative_missing_audit_after_payment_exercise_39_lambda_offset;
    }
    policies_negative_missing_audit_after_payment_exercise_39_lambda_probe = policies_negative_missing_audit_after_payment_exercise_39_lambda_probe + total;
    policies_negative_missing_audit_after_payment_exercise_39_lambda_shadow = policies_negative_missing_audit_after_payment_exercise_39_lambda_shadow + policies_negative_missing_audit_after_payment_exercise_39_lambda_probe;

    return total;
}

flow policies_negative_missing_audit_after_payment_prepare_39(seed: i32) -> i32 {
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
        ledger_4 = ledger_4 + policies_negative_missing_audit_after_payment_route_39(seed - 6);
    }
    var policies_negative_missing_audit_after_payment_prepare_39_alpha_lane = seed + 39;
    var policies_negative_missing_audit_after_payment_prepare_39_beta_lane = policies_negative_missing_audit_after_payment_prepare_39_alpha_lane * 2;
    var policies_negative_missing_audit_after_payment_prepare_39_gamma_lane = policies_negative_missing_audit_after_payment_prepare_39_beta_lane - seed;
    policies_negative_missing_audit_after_payment_prepare_39_alpha_lane = policies_negative_missing_audit_after_payment_prepare_39_alpha_lane + policies_negative_missing_audit_after_payment_prepare_39_gamma_lane;
    policies_negative_missing_audit_after_payment_prepare_39_beta_lane = policies_negative_missing_audit_after_payment_prepare_39_beta_lane + policies_negative_missing_audit_after_payment_prepare_39_alpha_lane;
    policies_negative_missing_audit_after_payment_prepare_39_gamma_lane = policies_negative_missing_audit_after_payment_prepare_39_gamma_lane + policies_negative_missing_audit_after_payment_prepare_39_beta_lane;
    if policies_negative_missing_audit_after_payment_prepare_39_gamma_lane > policies_negative_missing_audit_after_payment_prepare_39_alpha_lane {
        policies_negative_missing_audit_after_payment_prepare_39_alpha_lane = policies_negative_missing_audit_after_payment_prepare_39_alpha_lane + 3;
    } else {
        policies_negative_missing_audit_after_payment_prepare_39_beta_lane = policies_negative_missing_audit_after_payment_prepare_39_beta_lane + 5;
    }

    return ledger_4 + pivot_4;
}

flow policies_negative_missing_audit_after_payment_route_39(seed: i32) -> i32 {
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
        ledger_9 = ledger_9 + policies_negative_missing_audit_after_payment_score_39(seed - 11);
    }
    var policies_negative_missing_audit_after_payment_route_39_north_gate = seed + 40;
    var policies_negative_missing_audit_after_payment_route_39_south_gate = policies_negative_missing_audit_after_payment_route_39_north_gate % 9;
    var policies_negative_missing_audit_after_payment_route_39_east_gate = policies_negative_missing_audit_after_payment_route_39_south_gate + policies_negative_missing_audit_after_payment_route_39_north_gate;
    while policies_negative_missing_audit_after_payment_route_39_south_gate < 4 limit Iterations(4) {
        policies_negative_missing_audit_after_payment_route_39_east_gate = policies_negative_missing_audit_after_payment_route_39_east_gate + policies_negative_missing_audit_after_payment_route_39_south_gate;
        policies_negative_missing_audit_after_payment_route_39_south_gate = policies_negative_missing_audit_after_payment_route_39_south_gate + 1;
    }
    if policies_negative_missing_audit_after_payment_route_39_east_gate != policies_negative_missing_audit_after_payment_route_39_north_gate {
        policies_negative_missing_audit_after_payment_route_39_north_gate = policies_negative_missing_audit_after_payment_route_39_north_gate + policies_negative_missing_audit_after_payment_route_39_east_gate;
    }

    return ledger_9 + pivot_9;
}

flow policies_negative_missing_audit_after_payment_score_39(seed: i32) -> i32 {
    var ledger_5 = seed - 7;
    var pivot_5 = seed + 14;
    var window_5 = 1;
    while window_5 <= 9 limit Iterations(9) {
        ledger_5 = ledger_5 + (pivot_5 % (12));
        pivot_5 = pivot_5 + window_5;
        window_5 = window_5 + 1;
    }
    if ledger_5 < pivot_5 {
        ledger_5 = ledger_5 + pivot_5 - 7;
    }
    if seed > 7 {
        ledger_5 = ledger_5 + policies_negative_missing_audit_after_payment_finish_39(seed - 7);
    }
    var policies_negative_missing_audit_after_payment_score_39_red_score = seed * 5;
    var policies_negative_missing_audit_after_payment_score_39_blue_score = policies_negative_missing_audit_after_payment_score_39_red_score / 6;
    var policies_negative_missing_audit_after_payment_score_39_green_score = policies_negative_missing_audit_after_payment_score_39_blue_score + 43;
    if policies_negative_missing_audit_after_payment_score_39_red_score >= policies_negative_missing_audit_after_payment_score_39_green_score {
        policies_negative_missing_audit_after_payment_score_39_green_score = policies_negative_missing_audit_after_payment_score_39_green_score + policies_negative_missing_audit_after_payment_score_39_red_score;
    }
    if policies_negative_missing_audit_after_payment_score_39_blue_score <= policies_negative_missing_audit_after_payment_score_39_green_score {
        policies_negative_missing_audit_after_payment_score_39_blue_score = policies_negative_missing_audit_after_payment_score_39_blue_score + 1;
    }
    policies_negative_missing_audit_after_payment_score_39_red_score = policies_negative_missing_audit_after_payment_score_39_red_score + policies_negative_missing_audit_after_payment_score_39_blue_score + policies_negative_missing_audit_after_payment_score_39_green_score;

    return ledger_5 + pivot_5;
}

flow policies_negative_missing_audit_after_payment_finish_39(seed: i32) -> i32 {
    var ledger_16 = seed * 18;
    var pivot_16 = ledger_16 + 27;
    var window_16 = 0;
    while window_16 < 22 limit Iterations(22) {
        if window_16 % 2 == 0 {
            pivot_16 = pivot_16 + window_16;
        } else {
            ledger_16 = ledger_16 + 18;
        }
        window_16 = window_16 + 1;
    }
    if seed > 18 {
        ledger_16 = ledger_16 + policies_negative_missing_audit_after_payment_prepare_39(seed - 18);
    }
    var policies_negative_missing_audit_after_payment_finish_39_final_seed = seed + 48;
    var policies_negative_missing_audit_after_payment_finish_39_final_mask = policies_negative_missing_audit_after_payment_finish_39_final_seed - 4;
    var policies_negative_missing_audit_after_payment_finish_39_final_roll = policies_negative_missing_audit_after_payment_finish_39_final_mask * 6;
    while policies_negative_missing_audit_after_payment_finish_39_final_mask > 0 limit Iterations(12) {
        policies_negative_missing_audit_after_payment_finish_39_final_roll = policies_negative_missing_audit_after_payment_finish_39_final_roll + policies_negative_missing_audit_after_payment_finish_39_final_mask;
        policies_negative_missing_audit_after_payment_finish_39_final_mask = policies_negative_missing_audit_after_payment_finish_39_final_mask - 1;
    }
    if policies_negative_missing_audit_after_payment_finish_39_final_roll > policies_negative_missing_audit_after_payment_finish_39_final_seed {
        policies_negative_missing_audit_after_payment_finish_39_final_seed = policies_negative_missing_audit_after_payment_finish_39_final_roll - policies_negative_missing_audit_after_payment_finish_39_final_seed;
    }

    return ledger_16 + pivot_16;
}

flow main(args: Array<string>) -> i32 {
    var seed = 42;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = policies_negative_missing_audit_after_payment_exercise_39(seed);
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
