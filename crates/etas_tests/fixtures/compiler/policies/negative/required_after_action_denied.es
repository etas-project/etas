module tests.compiler.policies.negative.required_after_action_denied;

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

spec FixturePolicy: trace = +Payment.charge & -Audit.write & (Audit.write << Payment.charge);

flow policies_negative_required_after_action_denied_exercise_42(seed: i32) -> i32
    ~ FixturePolicy
{
    var total = policies_negative_required_after_action_denied_prepare_42(seed);
    total = total + policies_negative_required_after_action_denied_route_42(seed + 2);
    if total < 0 {
        total = 0 - total;
    }
    let policies_negative_required_after_action_denied_adjust_42: i32 -> i32 = (value: i32) => value + 42;
    total = policies_negative_required_after_action_denied_adjust_42(total);
    let charge = perform Payment.charge("BillingAccount", 100);
    perform Audit.write("payment");
    total = total + charge.code;
    total = total + policies_negative_required_after_action_denied_score_42(2);
    total = total + policies_negative_required_after_action_denied_finish_42(1);
    if total > 4000 {
        total = total - 4000;
    } else {
        total = total + 73;
    }
    var policies_negative_required_after_action_denied_exercise_42_lambda_probe = seed + 55;
    var policies_negative_required_after_action_denied_exercise_42_lambda_shadow = policies_negative_required_after_action_denied_exercise_42_lambda_probe * 3;
    var policies_negative_required_after_action_denied_exercise_42_lambda_offset = policies_negative_required_after_action_denied_exercise_42_lambda_shadow - total;
    if policies_negative_required_after_action_denied_exercise_42_lambda_offset > 0 {
        total = total + policies_negative_required_after_action_denied_exercise_42_lambda_offset;
    } else {
        total = total - policies_negative_required_after_action_denied_exercise_42_lambda_offset;
    }
    policies_negative_required_after_action_denied_exercise_42_lambda_probe = policies_negative_required_after_action_denied_exercise_42_lambda_probe + total;
    policies_negative_required_after_action_denied_exercise_42_lambda_shadow = policies_negative_required_after_action_denied_exercise_42_lambda_shadow + policies_negative_required_after_action_denied_exercise_42_lambda_probe;

    return total;
}

flow policies_negative_required_after_action_denied_prepare_42(seed: i32) -> i32 {
    var ledger_7 = seed * 9;
    var pivot_7 = ledger_7 + 18;
    var window_7 = 0;
    while window_7 < 13 limit Iterations(13) {
        if window_7 % 2 == 0 {
            pivot_7 = pivot_7 + window_7;
        } else {
            ledger_7 = ledger_7 + 9;
        }
        window_7 = window_7 + 1;
    }
    if seed > 9 {
        ledger_7 = ledger_7 + policies_negative_required_after_action_denied_route_42(seed - 9);
    }
    var policies_negative_required_after_action_denied_prepare_42_alpha_lane = seed + 42;
    var policies_negative_required_after_action_denied_prepare_42_beta_lane = policies_negative_required_after_action_denied_prepare_42_alpha_lane * 2;
    var policies_negative_required_after_action_denied_prepare_42_gamma_lane = policies_negative_required_after_action_denied_prepare_42_beta_lane - seed;
    policies_negative_required_after_action_denied_prepare_42_alpha_lane = policies_negative_required_after_action_denied_prepare_42_alpha_lane + policies_negative_required_after_action_denied_prepare_42_gamma_lane;
    policies_negative_required_after_action_denied_prepare_42_beta_lane = policies_negative_required_after_action_denied_prepare_42_beta_lane + policies_negative_required_after_action_denied_prepare_42_alpha_lane;
    policies_negative_required_after_action_denied_prepare_42_gamma_lane = policies_negative_required_after_action_denied_prepare_42_gamma_lane + policies_negative_required_after_action_denied_prepare_42_beta_lane;
    if policies_negative_required_after_action_denied_prepare_42_gamma_lane > policies_negative_required_after_action_denied_prepare_42_alpha_lane {
        policies_negative_required_after_action_denied_prepare_42_alpha_lane = policies_negative_required_after_action_denied_prepare_42_alpha_lane + 3;
    } else {
        policies_negative_required_after_action_denied_prepare_42_beta_lane = policies_negative_required_after_action_denied_prepare_42_beta_lane + 5;
    }

    return ledger_7 + pivot_7;
}

flow policies_negative_required_after_action_denied_route_42(seed: i32) -> i32 {
    var ledger_12 = seed + 42;
    var pivot_12 = ledger_12 / 14;
    var window_12 = 14;
    while window_12 > 0 limit Iterations(15) {
        pivot_12 = pivot_12 + window_12;
        ledger_12 = ledger_12 + pivot_12;
        window_12 = window_12 - 1;
    }
    if ledger_12 != pivot_12 {
        ledger_12 = ledger_12 + 14;
    }
    if seed > 14 {
        ledger_12 = ledger_12 + policies_negative_required_after_action_denied_score_42(seed - 14);
    }
    var policies_negative_required_after_action_denied_route_42_north_gate = seed + 43;
    var policies_negative_required_after_action_denied_route_42_south_gate = policies_negative_required_after_action_denied_route_42_north_gate % 5;
    var policies_negative_required_after_action_denied_route_42_east_gate = policies_negative_required_after_action_denied_route_42_south_gate + policies_negative_required_after_action_denied_route_42_north_gate;
    while policies_negative_required_after_action_denied_route_42_south_gate < 4 limit Iterations(4) {
        policies_negative_required_after_action_denied_route_42_east_gate = policies_negative_required_after_action_denied_route_42_east_gate + policies_negative_required_after_action_denied_route_42_south_gate;
        policies_negative_required_after_action_denied_route_42_south_gate = policies_negative_required_after_action_denied_route_42_south_gate + 1;
    }
    if policies_negative_required_after_action_denied_route_42_east_gate != policies_negative_required_after_action_denied_route_42_north_gate {
        policies_negative_required_after_action_denied_route_42_north_gate = policies_negative_required_after_action_denied_route_42_north_gate + policies_negative_required_after_action_denied_route_42_east_gate;
    }

    return ledger_12 + pivot_12;
}

flow policies_negative_required_after_action_denied_score_42(seed: i32) -> i32 {
    var ledger_8 = seed + 10;
    var pivot_8 = ledger_8 * 11;
    var window_8 = 0;
    while window_8 < 13 limit Iterations(13) {
        pivot_8 = pivot_8 + window_8 + 10;
        window_8 = window_8 + 1;
    }
    if pivot_8 > 70 {
        ledger_8 = ledger_8 + pivot_8;
    } else {
        ledger_8 = ledger_8 - 10;
    }
    if seed > 10 {
        ledger_8 = ledger_8 + policies_negative_required_after_action_denied_finish_42(seed - 10);
    }
    var policies_negative_required_after_action_denied_score_42_red_score = seed * 8;
    var policies_negative_required_after_action_denied_score_42_blue_score = policies_negative_required_after_action_denied_score_42_red_score / 4;
    var policies_negative_required_after_action_denied_score_42_green_score = policies_negative_required_after_action_denied_score_42_blue_score + 46;
    if policies_negative_required_after_action_denied_score_42_red_score >= policies_negative_required_after_action_denied_score_42_green_score {
        policies_negative_required_after_action_denied_score_42_green_score = policies_negative_required_after_action_denied_score_42_green_score + policies_negative_required_after_action_denied_score_42_red_score;
    }
    if policies_negative_required_after_action_denied_score_42_blue_score <= policies_negative_required_after_action_denied_score_42_green_score {
        policies_negative_required_after_action_denied_score_42_blue_score = policies_negative_required_after_action_denied_score_42_blue_score + 1;
    }
    policies_negative_required_after_action_denied_score_42_red_score = policies_negative_required_after_action_denied_score_42_red_score + policies_negative_required_after_action_denied_score_42_blue_score + policies_negative_required_after_action_denied_score_42_green_score;

    return ledger_8 + pivot_8;
}

flow policies_negative_required_after_action_denied_finish_42(seed: i32) -> i32 {
    var ledger_19 = seed - 21;
    var pivot_19 = seed + 42;
    var window_19 = 1;
    while window_19 <= 23 limit Iterations(23) {
        ledger_19 = ledger_19 + (pivot_19 % (26));
        pivot_19 = pivot_19 + window_19;
        window_19 = window_19 + 1;
    }
    if ledger_19 < pivot_19 {
        ledger_19 = ledger_19 + pivot_19 - 21;
    }
    if seed > 21 {
        ledger_19 = ledger_19 + policies_negative_required_after_action_denied_prepare_42(seed - 21);
    }
    var policies_negative_required_after_action_denied_finish_42_final_seed = seed + 51;
    var policies_negative_required_after_action_denied_finish_42_final_mask = policies_negative_required_after_action_denied_finish_42_final_seed - 1;
    var policies_negative_required_after_action_denied_finish_42_final_roll = policies_negative_required_after_action_denied_finish_42_final_mask * 5;
    while policies_negative_required_after_action_denied_finish_42_final_mask > 0 limit Iterations(12) {
        policies_negative_required_after_action_denied_finish_42_final_roll = policies_negative_required_after_action_denied_finish_42_final_roll + policies_negative_required_after_action_denied_finish_42_final_mask;
        policies_negative_required_after_action_denied_finish_42_final_mask = policies_negative_required_after_action_denied_finish_42_final_mask - 1;
    }
    if policies_negative_required_after_action_denied_finish_42_final_roll > policies_negative_required_after_action_denied_finish_42_final_seed {
        policies_negative_required_after_action_denied_finish_42_final_seed = policies_negative_required_after_action_denied_finish_42_final_roll - policies_negative_required_after_action_denied_finish_42_final_seed;
    }

    return ledger_19 + pivot_19;
}

flow main(args: Array<string>) -> i32 {
    var seed = 45;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = policies_negative_required_after_action_denied_exercise_42(seed);
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
