module tests.compiler.policies.negative.approval_only_one_branch;

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

flow policies_negative_approval_only_one_branch_exercise_33(seed: i32) -> i32
    ~ FixturePolicy
{
    var total = policies_negative_approval_only_one_branch_prepare_33(seed);
    total = total + policies_negative_approval_only_one_branch_route_33(seed + 2);
    if total < 0 {
        total = 0 - total;
    }
    let policies_negative_approval_only_one_branch_adjust_33: i32 -> i32 = (value: i32) => value + 33;
    total = policies_negative_approval_only_one_branch_adjust_33(total);
    if seed > 10 {
        perform Approval.request("branch only");
    }
    perform Email.send("WorkAccount", "body");
    total = total + 10;
    total = total + policies_negative_approval_only_one_branch_score_33(2);
    total = total + policies_negative_approval_only_one_branch_finish_33(1);
    if total > 4000 {
        total = total - 4000;
    } else {
        total = total + 64;
    }
    var policies_negative_approval_only_one_branch_exercise_33_lambda_probe = seed + 46;
    var policies_negative_approval_only_one_branch_exercise_33_lambda_shadow = policies_negative_approval_only_one_branch_exercise_33_lambda_probe * 3;
    var policies_negative_approval_only_one_branch_exercise_33_lambda_offset = policies_negative_approval_only_one_branch_exercise_33_lambda_shadow - total;
    if policies_negative_approval_only_one_branch_exercise_33_lambda_offset > 0 {
        total = total + policies_negative_approval_only_one_branch_exercise_33_lambda_offset;
    } else {
        total = total - policies_negative_approval_only_one_branch_exercise_33_lambda_offset;
    }
    policies_negative_approval_only_one_branch_exercise_33_lambda_probe = policies_negative_approval_only_one_branch_exercise_33_lambda_probe + total;
    policies_negative_approval_only_one_branch_exercise_33_lambda_shadow = policies_negative_approval_only_one_branch_exercise_33_lambda_shadow + policies_negative_approval_only_one_branch_exercise_33_lambda_probe;

    return total;
}

flow policies_negative_approval_only_one_branch_prepare_33(seed: i32) -> i32 {
    var ledger_7 = seed - 9;
    var pivot_7 = seed + 18;
    var window_7 = 1;
    while window_7 <= 11 limit Iterations(11) {
        ledger_7 = ledger_7 + (pivot_7 % (14));
        pivot_7 = pivot_7 + window_7;
        window_7 = window_7 + 1;
    }
    if ledger_7 < pivot_7 {
        ledger_7 = ledger_7 + pivot_7 - 9;
    }
    if seed > 9 {
        ledger_7 = ledger_7 + policies_negative_approval_only_one_branch_route_33(seed - 9);
    }
    var policies_negative_approval_only_one_branch_prepare_33_alpha_lane = seed + 33;
    var policies_negative_approval_only_one_branch_prepare_33_beta_lane = policies_negative_approval_only_one_branch_prepare_33_alpha_lane * 2;
    var policies_negative_approval_only_one_branch_prepare_33_gamma_lane = policies_negative_approval_only_one_branch_prepare_33_beta_lane - seed;
    policies_negative_approval_only_one_branch_prepare_33_alpha_lane = policies_negative_approval_only_one_branch_prepare_33_alpha_lane + policies_negative_approval_only_one_branch_prepare_33_gamma_lane;
    policies_negative_approval_only_one_branch_prepare_33_beta_lane = policies_negative_approval_only_one_branch_prepare_33_beta_lane + policies_negative_approval_only_one_branch_prepare_33_alpha_lane;
    policies_negative_approval_only_one_branch_prepare_33_gamma_lane = policies_negative_approval_only_one_branch_prepare_33_gamma_lane + policies_negative_approval_only_one_branch_prepare_33_beta_lane;
    if policies_negative_approval_only_one_branch_prepare_33_gamma_lane > policies_negative_approval_only_one_branch_prepare_33_alpha_lane {
        policies_negative_approval_only_one_branch_prepare_33_alpha_lane = policies_negative_approval_only_one_branch_prepare_33_alpha_lane + 3;
    } else {
        policies_negative_approval_only_one_branch_prepare_33_beta_lane = policies_negative_approval_only_one_branch_prepare_33_beta_lane + 5;
    }

    return ledger_7 + pivot_7;
}

flow policies_negative_approval_only_one_branch_route_33(seed: i32) -> i32 {
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
        ledger_3 = ledger_3 + policies_negative_approval_only_one_branch_score_33(seed - 5);
    }
    var policies_negative_approval_only_one_branch_route_33_north_gate = seed + 34;
    var policies_negative_approval_only_one_branch_route_33_south_gate = policies_negative_approval_only_one_branch_route_33_north_gate % 10;
    var policies_negative_approval_only_one_branch_route_33_east_gate = policies_negative_approval_only_one_branch_route_33_south_gate + policies_negative_approval_only_one_branch_route_33_north_gate;
    while policies_negative_approval_only_one_branch_route_33_south_gate < 4 limit Iterations(4) {
        policies_negative_approval_only_one_branch_route_33_east_gate = policies_negative_approval_only_one_branch_route_33_east_gate + policies_negative_approval_only_one_branch_route_33_south_gate;
        policies_negative_approval_only_one_branch_route_33_south_gate = policies_negative_approval_only_one_branch_route_33_south_gate + 1;
    }
    if policies_negative_approval_only_one_branch_route_33_east_gate != policies_negative_approval_only_one_branch_route_33_north_gate {
        policies_negative_approval_only_one_branch_route_33_north_gate = policies_negative_approval_only_one_branch_route_33_north_gate + policies_negative_approval_only_one_branch_route_33_east_gate;
    }

    return ledger_3 + pivot_3;
}

flow policies_negative_approval_only_one_branch_score_33(seed: i32) -> i32 {
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
        ledger_12 = ledger_12 + policies_negative_approval_only_one_branch_finish_33(seed - 14);
    }
    var policies_negative_approval_only_one_branch_score_33_red_score = seed * 8;
    var policies_negative_approval_only_one_branch_score_33_blue_score = policies_negative_approval_only_one_branch_score_33_red_score / 5;
    var policies_negative_approval_only_one_branch_score_33_green_score = policies_negative_approval_only_one_branch_score_33_blue_score + 37;
    if policies_negative_approval_only_one_branch_score_33_red_score >= policies_negative_approval_only_one_branch_score_33_green_score {
        policies_negative_approval_only_one_branch_score_33_green_score = policies_negative_approval_only_one_branch_score_33_green_score + policies_negative_approval_only_one_branch_score_33_red_score;
    }
    if policies_negative_approval_only_one_branch_score_33_blue_score <= policies_negative_approval_only_one_branch_score_33_green_score {
        policies_negative_approval_only_one_branch_score_33_blue_score = policies_negative_approval_only_one_branch_score_33_blue_score + 1;
    }
    policies_negative_approval_only_one_branch_score_33_red_score = policies_negative_approval_only_one_branch_score_33_red_score + policies_negative_approval_only_one_branch_score_33_blue_score + policies_negative_approval_only_one_branch_score_33_green_score;

    return ledger_12 + pivot_12;
}

flow policies_negative_approval_only_one_branch_finish_33(seed: i32) -> i32 {
    var ledger_10 = seed + 12;
    var pivot_10 = ledger_10 * 13;
    var window_10 = 0;
    while window_10 < 15 limit Iterations(15) {
        pivot_10 = pivot_10 + window_10 + 12;
        window_10 = window_10 + 1;
    }
    if pivot_10 > 84 {
        ledger_10 = ledger_10 + pivot_10;
    } else {
        ledger_10 = ledger_10 - 12;
    }
    if seed > 12 {
        ledger_10 = ledger_10 + policies_negative_approval_only_one_branch_prepare_33(seed - 12);
    }
    var policies_negative_approval_only_one_branch_finish_33_final_seed = seed + 42;
    var policies_negative_approval_only_one_branch_finish_33_final_mask = policies_negative_approval_only_one_branch_finish_33_final_seed - 4;
    var policies_negative_approval_only_one_branch_finish_33_final_roll = policies_negative_approval_only_one_branch_finish_33_final_mask * 4;
    while policies_negative_approval_only_one_branch_finish_33_final_mask > 0 limit Iterations(12) {
        policies_negative_approval_only_one_branch_finish_33_final_roll = policies_negative_approval_only_one_branch_finish_33_final_roll + policies_negative_approval_only_one_branch_finish_33_final_mask;
        policies_negative_approval_only_one_branch_finish_33_final_mask = policies_negative_approval_only_one_branch_finish_33_final_mask - 1;
    }
    if policies_negative_approval_only_one_branch_finish_33_final_roll > policies_negative_approval_only_one_branch_finish_33_final_seed {
        policies_negative_approval_only_one_branch_finish_33_final_seed = policies_negative_approval_only_one_branch_finish_33_final_roll - policies_negative_approval_only_one_branch_finish_33_final_seed;
    }

    return ledger_10 + pivot_10;
}

flow main(args: Array<string>) -> i32 {
    var seed = 36;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = policies_negative_approval_only_one_branch_exercise_33(seed);
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
