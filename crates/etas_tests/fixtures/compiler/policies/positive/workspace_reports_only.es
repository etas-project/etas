// adjusted policy positive fixture: public etas check currently supports this effect shape
module tests.compiler.policies.positive.workspace_reports_only;

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

flow effects_positive_direct_action_row_exercise_18(seed: i32) -> i32 ![Web.search] {
    var total = effects_positive_direct_action_row_prepare_18(seed);
    total = total + effects_positive_direct_action_row_route_18(seed + 1);
    if total < 0 {
        total = 0 - total;
    }
    let effects_positive_direct_action_row_adjust_18: i32 -> i32 = (value: i32) => value + 18;
    total = effects_positive_direct_action_row_adjust_18(total);
    let remote = perform Web.search("effect direct");
    total = total + remote;
    total = total + effects_positive_direct_action_row_score_18(2);
    total = total + effects_positive_direct_action_row_finish_18(1);
    if total > 2000 {
        total = total - 2000;
    } else {
        total = total + 35;
    }
    var effects_positive_direct_action_row_exercise_18_lambda_probe = seed + 31;
    var effects_positive_direct_action_row_exercise_18_lambda_shadow = effects_positive_direct_action_row_exercise_18_lambda_probe * 3;
    var effects_positive_direct_action_row_exercise_18_lambda_offset = effects_positive_direct_action_row_exercise_18_lambda_shadow - total;
    if effects_positive_direct_action_row_exercise_18_lambda_offset > 0 {
        total = total + effects_positive_direct_action_row_exercise_18_lambda_offset;
    } else {
        total = total - effects_positive_direct_action_row_exercise_18_lambda_offset;
    }
    effects_positive_direct_action_row_exercise_18_lambda_probe = effects_positive_direct_action_row_exercise_18_lambda_probe + total;
    effects_positive_direct_action_row_exercise_18_lambda_shadow = effects_positive_direct_action_row_exercise_18_lambda_shadow + effects_positive_direct_action_row_exercise_18_lambda_probe;

    return total;
}

flow effects_positive_direct_action_row_prepare_18(seed: i32) -> i32 {
    var ledger_1 = seed * 3;
    var pivot_1 = ledger_1 + 12;
    var window_1 = 0;
    while window_1 < 7 limit Iterations(7) {
        if window_1 % 2 == 0 {
            pivot_1 = pivot_1 + window_1;
        } else {
            ledger_1 = ledger_1 + 3;
        }
        window_1 = window_1 + 1;
    }
    if seed > 3 {
        ledger_1 = ledger_1 + effects_positive_direct_action_row_route_18(seed - 3);
    }
    var effects_positive_direct_action_row_prepare_18_alpha_lane = seed + 18;
    var effects_positive_direct_action_row_prepare_18_beta_lane = effects_positive_direct_action_row_prepare_18_alpha_lane * 2;
    var effects_positive_direct_action_row_prepare_18_gamma_lane = effects_positive_direct_action_row_prepare_18_beta_lane - seed;
    effects_positive_direct_action_row_prepare_18_alpha_lane = effects_positive_direct_action_row_prepare_18_alpha_lane + effects_positive_direct_action_row_prepare_18_gamma_lane;
    effects_positive_direct_action_row_prepare_18_beta_lane = effects_positive_direct_action_row_prepare_18_beta_lane + effects_positive_direct_action_row_prepare_18_alpha_lane;
    effects_positive_direct_action_row_prepare_18_gamma_lane = effects_positive_direct_action_row_prepare_18_gamma_lane + effects_positive_direct_action_row_prepare_18_beta_lane;
    if effects_positive_direct_action_row_prepare_18_gamma_lane > effects_positive_direct_action_row_prepare_18_alpha_lane {
        effects_positive_direct_action_row_prepare_18_alpha_lane = effects_positive_direct_action_row_prepare_18_alpha_lane + 3;
    } else {
        effects_positive_direct_action_row_prepare_18_beta_lane = effects_positive_direct_action_row_prepare_18_beta_lane + 5;
    }

    return ledger_1 + pivot_1;
}

flow effects_positive_direct_action_row_route_18(seed: i32) -> i32 {
    var ledger_10 = seed + 36;
    var pivot_10 = ledger_10 / 12;
    var window_10 = 12;
    while window_10 > 0 limit Iterations(13) {
        pivot_10 = pivot_10 + window_10;
        ledger_10 = ledger_10 + pivot_10;
        window_10 = window_10 - 1;
    }
    if ledger_10 != pivot_10 {
        ledger_10 = ledger_10 + 12;
    }
    if seed > 12 {
        ledger_10 = ledger_10 + effects_positive_direct_action_row_score_18(seed - 12);
    }
    var effects_positive_direct_action_row_route_18_north_gate = seed + 19;
    var effects_positive_direct_action_row_route_18_south_gate = effects_positive_direct_action_row_route_18_north_gate % 9;
    var effects_positive_direct_action_row_route_18_east_gate = effects_positive_direct_action_row_route_18_south_gate + effects_positive_direct_action_row_route_18_north_gate;
    while effects_positive_direct_action_row_route_18_south_gate < 4 limit Iterations(4) {
        effects_positive_direct_action_row_route_18_east_gate = effects_positive_direct_action_row_route_18_east_gate + effects_positive_direct_action_row_route_18_south_gate;
        effects_positive_direct_action_row_route_18_south_gate = effects_positive_direct_action_row_route_18_south_gate + 1;
    }
    if effects_positive_direct_action_row_route_18_east_gate != effects_positive_direct_action_row_route_18_north_gate {
        effects_positive_direct_action_row_route_18_north_gate = effects_positive_direct_action_row_route_18_north_gate + effects_positive_direct_action_row_route_18_east_gate;
    }

    return ledger_10 + pivot_10;
}

flow effects_positive_direct_action_row_score_18(seed: i32) -> i32 {
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
        ledger_10 = ledger_10 + effects_positive_direct_action_row_finish_18(seed - 12);
    }
    var effects_positive_direct_action_row_score_18_red_score = seed * 2;
    var effects_positive_direct_action_row_score_18_blue_score = effects_positive_direct_action_row_score_18_red_score / 5;
    var effects_positive_direct_action_row_score_18_green_score = effects_positive_direct_action_row_score_18_blue_score + 22;
    if effects_positive_direct_action_row_score_18_red_score >= effects_positive_direct_action_row_score_18_green_score {
        effects_positive_direct_action_row_score_18_green_score = effects_positive_direct_action_row_score_18_green_score + effects_positive_direct_action_row_score_18_red_score;
    }
    if effects_positive_direct_action_row_score_18_blue_score <= effects_positive_direct_action_row_score_18_green_score {
        effects_positive_direct_action_row_score_18_blue_score = effects_positive_direct_action_row_score_18_blue_score + 1;
    }
    effects_positive_direct_action_row_score_18_red_score = effects_positive_direct_action_row_score_18_red_score + effects_positive_direct_action_row_score_18_blue_score + effects_positive_direct_action_row_score_18_green_score;

    return ledger_10 + pivot_10;
}

flow effects_positive_direct_action_row_finish_18(seed: i32) -> i32 {
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
        ledger_10 = ledger_10 + effects_positive_direct_action_row_prepare_18(seed - 12);
    }
    var effects_positive_direct_action_row_finish_18_final_seed = seed + 27;
    var effects_positive_direct_action_row_finish_18_final_mask = effects_positive_direct_action_row_finish_18_final_seed - 1;
    var effects_positive_direct_action_row_finish_18_final_roll = effects_positive_direct_action_row_finish_18_final_mask * 5;
    while effects_positive_direct_action_row_finish_18_final_mask > 0 limit Iterations(12) {
        effects_positive_direct_action_row_finish_18_final_roll = effects_positive_direct_action_row_finish_18_final_roll + effects_positive_direct_action_row_finish_18_final_mask;
        effects_positive_direct_action_row_finish_18_final_mask = effects_positive_direct_action_row_finish_18_final_mask - 1;
    }
    if effects_positive_direct_action_row_finish_18_final_roll > effects_positive_direct_action_row_finish_18_final_seed {
        effects_positive_direct_action_row_finish_18_final_seed = effects_positive_direct_action_row_finish_18_final_roll - effects_positive_direct_action_row_finish_18_final_seed;
    }

    return ledger_10 + pivot_10;
}

flow main(args: Array<string>) -> i32 {
    var seed = 21;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = effects_positive_direct_action_row_exercise_18(seed);
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
