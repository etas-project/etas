module tests.compiler.policies.negative.workspace_path_not_allowed;

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

spec FixturePolicy: trace = +Log.write;

flow policies_negative_workspace_path_not_allowed_exercise_45(seed: i32) -> i32
    ~ FixturePolicy
{
    var total = policies_negative_workspace_path_not_allowed_prepare_45(seed);
    total = total + policies_negative_workspace_path_not_allowed_route_45(seed + 2);
    if total < 0 {
        total = 0 - total;
    }
    let policies_negative_workspace_path_not_allowed_adjust_45: i32 -> i32 = (value: i32) => value + 45;
    total = policies_negative_workspace_path_not_allowed_adjust_45(total);
    perform Workspace.write("tmp/**", "tmp/out.md", "body");
    total = total + 9;
    total = total + policies_negative_workspace_path_not_allowed_score_45(2);
    total = total + policies_negative_workspace_path_not_allowed_finish_45(1);
    if total > 4000 {
        total = total - 4000;
    } else {
        total = total + 76;
    }
    var policies_negative_workspace_path_not_allowed_exercise_45_lambda_probe = seed + 58;
    var policies_negative_workspace_path_not_allowed_exercise_45_lambda_shadow = policies_negative_workspace_path_not_allowed_exercise_45_lambda_probe * 3;
    var policies_negative_workspace_path_not_allowed_exercise_45_lambda_offset = policies_negative_workspace_path_not_allowed_exercise_45_lambda_shadow - total;
    if policies_negative_workspace_path_not_allowed_exercise_45_lambda_offset > 0 {
        total = total + policies_negative_workspace_path_not_allowed_exercise_45_lambda_offset;
    } else {
        total = total - policies_negative_workspace_path_not_allowed_exercise_45_lambda_offset;
    }
    policies_negative_workspace_path_not_allowed_exercise_45_lambda_probe = policies_negative_workspace_path_not_allowed_exercise_45_lambda_probe + total;
    policies_negative_workspace_path_not_allowed_exercise_45_lambda_shadow = policies_negative_workspace_path_not_allowed_exercise_45_lambda_shadow + policies_negative_workspace_path_not_allowed_exercise_45_lambda_probe;

    return total;
}

flow policies_negative_workspace_path_not_allowed_prepare_45(seed: i32) -> i32 {
    var ledger_1 = seed - 3;
    var pivot_1 = seed + 6;
    var window_1 = 1;
    while window_1 <= 5 limit Iterations(5) {
        ledger_1 = ledger_1 + (pivot_1 % (8));
        pivot_1 = pivot_1 + window_1;
        window_1 = window_1 + 1;
    }
    if ledger_1 < pivot_1 {
        ledger_1 = ledger_1 + pivot_1 - 3;
    }
    if seed > 3 {
        ledger_1 = ledger_1 + policies_negative_workspace_path_not_allowed_route_45(seed - 3);
    }
    var policies_negative_workspace_path_not_allowed_prepare_45_alpha_lane = seed + 45;
    var policies_negative_workspace_path_not_allowed_prepare_45_beta_lane = policies_negative_workspace_path_not_allowed_prepare_45_alpha_lane * 2;
    var policies_negative_workspace_path_not_allowed_prepare_45_gamma_lane = policies_negative_workspace_path_not_allowed_prepare_45_beta_lane - seed;
    policies_negative_workspace_path_not_allowed_prepare_45_alpha_lane = policies_negative_workspace_path_not_allowed_prepare_45_alpha_lane + policies_negative_workspace_path_not_allowed_prepare_45_gamma_lane;
    policies_negative_workspace_path_not_allowed_prepare_45_beta_lane = policies_negative_workspace_path_not_allowed_prepare_45_beta_lane + policies_negative_workspace_path_not_allowed_prepare_45_alpha_lane;
    policies_negative_workspace_path_not_allowed_prepare_45_gamma_lane = policies_negative_workspace_path_not_allowed_prepare_45_gamma_lane + policies_negative_workspace_path_not_allowed_prepare_45_beta_lane;
    if policies_negative_workspace_path_not_allowed_prepare_45_gamma_lane > policies_negative_workspace_path_not_allowed_prepare_45_alpha_lane {
        policies_negative_workspace_path_not_allowed_prepare_45_alpha_lane = policies_negative_workspace_path_not_allowed_prepare_45_alpha_lane + 3;
    } else {
        policies_negative_workspace_path_not_allowed_prepare_45_beta_lane = policies_negative_workspace_path_not_allowed_prepare_45_beta_lane + 5;
    }

    return ledger_1 + pivot_1;
}

flow policies_negative_workspace_path_not_allowed_route_45(seed: i32) -> i32 {
    var ledger_4 = seed * 6;
    var pivot_4 = ledger_4 + 15;
    var window_4 = 0;
    while window_4 < 10 limit Iterations(10) {
        if window_4 % 2 == 0 {
            pivot_4 = pivot_4 + window_4;
        } else {
            ledger_4 = ledger_4 + 6;
        }
        window_4 = window_4 + 1;
    }
    if seed > 6 {
        ledger_4 = ledger_4 + policies_negative_workspace_path_not_allowed_score_45(seed - 6);
    }
    var policies_negative_workspace_path_not_allowed_route_45_north_gate = seed + 46;
    var policies_negative_workspace_path_not_allowed_route_45_south_gate = policies_negative_workspace_path_not_allowed_route_45_north_gate % 8;
    var policies_negative_workspace_path_not_allowed_route_45_east_gate = policies_negative_workspace_path_not_allowed_route_45_south_gate + policies_negative_workspace_path_not_allowed_route_45_north_gate;
    while policies_negative_workspace_path_not_allowed_route_45_south_gate < 4 limit Iterations(4) {
        policies_negative_workspace_path_not_allowed_route_45_east_gate = policies_negative_workspace_path_not_allowed_route_45_east_gate + policies_negative_workspace_path_not_allowed_route_45_south_gate;
        policies_negative_workspace_path_not_allowed_route_45_south_gate = policies_negative_workspace_path_not_allowed_route_45_south_gate + 1;
    }
    if policies_negative_workspace_path_not_allowed_route_45_east_gate != policies_negative_workspace_path_not_allowed_route_45_north_gate {
        policies_negative_workspace_path_not_allowed_route_45_north_gate = policies_negative_workspace_path_not_allowed_route_45_north_gate + policies_negative_workspace_path_not_allowed_route_45_east_gate;
    }

    return ledger_4 + pivot_4;
}

flow policies_negative_workspace_path_not_allowed_score_45(seed: i32) -> i32 {
    var ledger_11 = seed + 39;
    var pivot_11 = ledger_11 / 13;
    var window_11 = 13;
    while window_11 > 0 limit Iterations(14) {
        pivot_11 = pivot_11 + window_11;
        ledger_11 = ledger_11 + pivot_11;
        window_11 = window_11 - 1;
    }
    if ledger_11 != pivot_11 {
        ledger_11 = ledger_11 + 13;
    }
    if seed > 13 {
        ledger_11 = ledger_11 + policies_negative_workspace_path_not_allowed_finish_45(seed - 13);
    }
    var policies_negative_workspace_path_not_allowed_score_45_red_score = seed * 2;
    var policies_negative_workspace_path_not_allowed_score_45_blue_score = policies_negative_workspace_path_not_allowed_score_45_red_score / 2;
    var policies_negative_workspace_path_not_allowed_score_45_green_score = policies_negative_workspace_path_not_allowed_score_45_blue_score + 49;
    if policies_negative_workspace_path_not_allowed_score_45_red_score >= policies_negative_workspace_path_not_allowed_score_45_green_score {
        policies_negative_workspace_path_not_allowed_score_45_green_score = policies_negative_workspace_path_not_allowed_score_45_green_score + policies_negative_workspace_path_not_allowed_score_45_red_score;
    }
    if policies_negative_workspace_path_not_allowed_score_45_blue_score <= policies_negative_workspace_path_not_allowed_score_45_green_score {
        policies_negative_workspace_path_not_allowed_score_45_blue_score = policies_negative_workspace_path_not_allowed_score_45_blue_score + 1;
    }
    policies_negative_workspace_path_not_allowed_score_45_red_score = policies_negative_workspace_path_not_allowed_score_45_red_score + policies_negative_workspace_path_not_allowed_score_45_blue_score + policies_negative_workspace_path_not_allowed_score_45_green_score;

    return ledger_11 + pivot_11;
}

flow policies_negative_workspace_path_not_allowed_finish_45(seed: i32) -> i32 {
    var ledger_7 = seed + 9;
    var pivot_7 = ledger_7 * 10;
    var window_7 = 0;
    while window_7 < 12 limit Iterations(12) {
        pivot_7 = pivot_7 + window_7 + 9;
        window_7 = window_7 + 1;
    }
    if pivot_7 > 63 {
        ledger_7 = ledger_7 + pivot_7;
    } else {
        ledger_7 = ledger_7 - 9;
    }
    if seed > 9 {
        ledger_7 = ledger_7 + policies_negative_workspace_path_not_allowed_prepare_45(seed - 9);
    }
    var policies_negative_workspace_path_not_allowed_finish_45_final_seed = seed + 54;
    var policies_negative_workspace_path_not_allowed_finish_45_final_mask = policies_negative_workspace_path_not_allowed_finish_45_final_seed - 4;
    var policies_negative_workspace_path_not_allowed_finish_45_final_roll = policies_negative_workspace_path_not_allowed_finish_45_final_mask * 4;
    while policies_negative_workspace_path_not_allowed_finish_45_final_mask > 0 limit Iterations(12) {
        policies_negative_workspace_path_not_allowed_finish_45_final_roll = policies_negative_workspace_path_not_allowed_finish_45_final_roll + policies_negative_workspace_path_not_allowed_finish_45_final_mask;
        policies_negative_workspace_path_not_allowed_finish_45_final_mask = policies_negative_workspace_path_not_allowed_finish_45_final_mask - 1;
    }
    if policies_negative_workspace_path_not_allowed_finish_45_final_roll > policies_negative_workspace_path_not_allowed_finish_45_final_seed {
        policies_negative_workspace_path_not_allowed_finish_45_final_seed = policies_negative_workspace_path_not_allowed_finish_45_final_roll - policies_negative_workspace_path_not_allowed_finish_45_final_seed;
    }

    return ledger_7 + pivot_7;
}

flow main(args: Array<string>) -> i32 {
    var seed = 48;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = policies_negative_workspace_path_not_allowed_exercise_45(seed);
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
