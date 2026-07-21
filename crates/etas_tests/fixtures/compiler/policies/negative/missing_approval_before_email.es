module tests.compiler.policies.negative.missing_approval_before_email;

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

flow policies_negative_missing_approval_before_email_exercise_38(seed: i32) -> i32
    ~ FixturePolicy
{
    var total = policies_negative_missing_approval_before_email_prepare_38(seed);
    total = total + policies_negative_missing_approval_before_email_route_38(seed + 2);
    if total < 0 {
        total = 0 - total;
    }
    let policies_negative_missing_approval_before_email_adjust_38: i32 -> i32 = (value: i32) => value + 38;
    total = policies_negative_missing_approval_before_email_adjust_38(total);
    let receipt = perform Email.send("WorkAccount", "body");
    total = total + receipt.code;
    total = total + policies_negative_missing_approval_before_email_score_38(2);
    total = total + policies_negative_missing_approval_before_email_finish_38(1);
    if total > 4000 {
        total = total - 4000;
    } else {
        total = total + 69;
    }
    var policies_negative_missing_approval_before_email_exercise_38_lambda_probe = seed + 51;
    var policies_negative_missing_approval_before_email_exercise_38_lambda_shadow = policies_negative_missing_approval_before_email_exercise_38_lambda_probe * 3;
    var policies_negative_missing_approval_before_email_exercise_38_lambda_offset = policies_negative_missing_approval_before_email_exercise_38_lambda_shadow - total;
    if policies_negative_missing_approval_before_email_exercise_38_lambda_offset > 0 {
        total = total + policies_negative_missing_approval_before_email_exercise_38_lambda_offset;
    } else {
        total = total - policies_negative_missing_approval_before_email_exercise_38_lambda_offset;
    }
    policies_negative_missing_approval_before_email_exercise_38_lambda_probe = policies_negative_missing_approval_before_email_exercise_38_lambda_probe + total;
    policies_negative_missing_approval_before_email_exercise_38_lambda_shadow = policies_negative_missing_approval_before_email_exercise_38_lambda_shadow + policies_negative_missing_approval_before_email_exercise_38_lambda_probe;

    return total;
}

flow policies_negative_missing_approval_before_email_prepare_38(seed: i32) -> i32 {
    var ledger_3 = seed * 5;
    var pivot_3 = ledger_3 + 14;
    var window_3 = 0;
    while window_3 < 9 limit Iterations(9) {
        if window_3 % 2 == 0 {
            pivot_3 = pivot_3 + window_3;
        } else {
            ledger_3 = ledger_3 + 5;
        }
        window_3 = window_3 + 1;
    }
    if seed > 5 {
        ledger_3 = ledger_3 + policies_negative_missing_approval_before_email_route_38(seed - 5);
    }
    var policies_negative_missing_approval_before_email_prepare_38_alpha_lane = seed + 38;
    var policies_negative_missing_approval_before_email_prepare_38_beta_lane = policies_negative_missing_approval_before_email_prepare_38_alpha_lane * 2;
    var policies_negative_missing_approval_before_email_prepare_38_gamma_lane = policies_negative_missing_approval_before_email_prepare_38_beta_lane - seed;
    policies_negative_missing_approval_before_email_prepare_38_alpha_lane = policies_negative_missing_approval_before_email_prepare_38_alpha_lane + policies_negative_missing_approval_before_email_prepare_38_gamma_lane;
    policies_negative_missing_approval_before_email_prepare_38_beta_lane = policies_negative_missing_approval_before_email_prepare_38_beta_lane + policies_negative_missing_approval_before_email_prepare_38_alpha_lane;
    policies_negative_missing_approval_before_email_prepare_38_gamma_lane = policies_negative_missing_approval_before_email_prepare_38_gamma_lane + policies_negative_missing_approval_before_email_prepare_38_beta_lane;
    if policies_negative_missing_approval_before_email_prepare_38_gamma_lane > policies_negative_missing_approval_before_email_prepare_38_alpha_lane {
        policies_negative_missing_approval_before_email_prepare_38_alpha_lane = policies_negative_missing_approval_before_email_prepare_38_alpha_lane + 3;
    } else {
        policies_negative_missing_approval_before_email_prepare_38_beta_lane = policies_negative_missing_approval_before_email_prepare_38_beta_lane + 5;
    }

    return ledger_3 + pivot_3;
}

flow policies_negative_missing_approval_before_email_route_38(seed: i32) -> i32 {
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
        ledger_8 = ledger_8 + policies_negative_missing_approval_before_email_score_38(seed - 10);
    }
    var policies_negative_missing_approval_before_email_route_38_north_gate = seed + 39;
    var policies_negative_missing_approval_before_email_route_38_south_gate = policies_negative_missing_approval_before_email_route_38_north_gate % 8;
    var policies_negative_missing_approval_before_email_route_38_east_gate = policies_negative_missing_approval_before_email_route_38_south_gate + policies_negative_missing_approval_before_email_route_38_north_gate;
    while policies_negative_missing_approval_before_email_route_38_south_gate < 4 limit Iterations(4) {
        policies_negative_missing_approval_before_email_route_38_east_gate = policies_negative_missing_approval_before_email_route_38_east_gate + policies_negative_missing_approval_before_email_route_38_south_gate;
        policies_negative_missing_approval_before_email_route_38_south_gate = policies_negative_missing_approval_before_email_route_38_south_gate + 1;
    }
    if policies_negative_missing_approval_before_email_route_38_east_gate != policies_negative_missing_approval_before_email_route_38_north_gate {
        policies_negative_missing_approval_before_email_route_38_north_gate = policies_negative_missing_approval_before_email_route_38_north_gate + policies_negative_missing_approval_before_email_route_38_east_gate;
    }

    return ledger_8 + pivot_8;
}

flow policies_negative_missing_approval_before_email_score_38(seed: i32) -> i32 {
    var ledger_17 = seed + 19;
    var pivot_17 = ledger_17 * 20;
    var window_17 = 0;
    while window_17 < 22 limit Iterations(22) {
        pivot_17 = pivot_17 + window_17 + 19;
        window_17 = window_17 + 1;
    }
    if pivot_17 > 133 {
        ledger_17 = ledger_17 + pivot_17;
    } else {
        ledger_17 = ledger_17 - 19;
    }
    if seed > 19 {
        ledger_17 = ledger_17 + policies_negative_missing_approval_before_email_finish_38(seed - 19);
    }
    var policies_negative_missing_approval_before_email_score_38_red_score = seed * 4;
    var policies_negative_missing_approval_before_email_score_38_blue_score = policies_negative_missing_approval_before_email_score_38_red_score / 5;
    var policies_negative_missing_approval_before_email_score_38_green_score = policies_negative_missing_approval_before_email_score_38_blue_score + 42;
    if policies_negative_missing_approval_before_email_score_38_red_score >= policies_negative_missing_approval_before_email_score_38_green_score {
        policies_negative_missing_approval_before_email_score_38_green_score = policies_negative_missing_approval_before_email_score_38_green_score + policies_negative_missing_approval_before_email_score_38_red_score;
    }
    if policies_negative_missing_approval_before_email_score_38_blue_score <= policies_negative_missing_approval_before_email_score_38_green_score {
        policies_negative_missing_approval_before_email_score_38_blue_score = policies_negative_missing_approval_before_email_score_38_blue_score + 1;
    }
    policies_negative_missing_approval_before_email_score_38_red_score = policies_negative_missing_approval_before_email_score_38_red_score + policies_negative_missing_approval_before_email_score_38_blue_score + policies_negative_missing_approval_before_email_score_38_green_score;

    return ledger_17 + pivot_17;
}

flow policies_negative_missing_approval_before_email_finish_38(seed: i32) -> i32 {
    var ledger_15 = seed - 17;
    var pivot_15 = seed + 34;
    var window_15 = 1;
    while window_15 <= 19 limit Iterations(19) {
        ledger_15 = ledger_15 + (pivot_15 % (22));
        pivot_15 = pivot_15 + window_15;
        window_15 = window_15 + 1;
    }
    if ledger_15 < pivot_15 {
        ledger_15 = ledger_15 + pivot_15 - 17;
    }
    if seed > 17 {
        ledger_15 = ledger_15 + policies_negative_missing_approval_before_email_prepare_38(seed - 17);
    }
    var policies_negative_missing_approval_before_email_finish_38_final_seed = seed + 47;
    var policies_negative_missing_approval_before_email_finish_38_final_mask = policies_negative_missing_approval_before_email_finish_38_final_seed - 3;
    var policies_negative_missing_approval_before_email_finish_38_final_roll = policies_negative_missing_approval_before_email_finish_38_final_mask * 5;
    while policies_negative_missing_approval_before_email_finish_38_final_mask > 0 limit Iterations(12) {
        policies_negative_missing_approval_before_email_finish_38_final_roll = policies_negative_missing_approval_before_email_finish_38_final_roll + policies_negative_missing_approval_before_email_finish_38_final_mask;
        policies_negative_missing_approval_before_email_finish_38_final_mask = policies_negative_missing_approval_before_email_finish_38_final_mask - 1;
    }
    if policies_negative_missing_approval_before_email_finish_38_final_roll > policies_negative_missing_approval_before_email_finish_38_final_seed {
        policies_negative_missing_approval_before_email_finish_38_final_seed = policies_negative_missing_approval_before_email_finish_38_final_roll - policies_negative_missing_approval_before_email_finish_38_final_seed;
    }

    return ledger_15 + pivot_15;
}

flow main(args: Array<string>) -> i32 {
    var seed = 41;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = policies_negative_missing_approval_before_email_exercise_38(seed);
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
