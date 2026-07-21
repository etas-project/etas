// adjusted policy positive fixture: public etas check currently supports this effect shape
module tests.compiler.policies.positive.payment_requires_approval_and_audit;

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

flow effects_positive_error_handler_recovers_exercise_20(seed: i32) -> i32 ![] {
    var total = effects_positive_error_handler_recovers_prepare_20(seed);
    total = total + effects_positive_error_handler_recovers_route_20(seed + 1);
    if total < 0 {
        total = 0 - total;
    }
    let effects_positive_error_handler_recovers_adjust_20: i32 -> i32 = (value: i32) => value + 20;
    total = effects_positive_error_handler_recovers_adjust_20(total);
    total = if total > 0 {
        total
    } else {
        1
    };
    total = total + effects_positive_error_handler_recovers_score_20(2);
    total = total + effects_positive_error_handler_recovers_finish_20(1);
    if total > 2000 {
        total = total - 2000;
    } else {
        total = total + 37;
    }
    var effects_positive_error_handler_recovers_exercise_20_lambda_probe = seed + 33;
    var effects_positive_error_handler_recovers_exercise_20_lambda_shadow = effects_positive_error_handler_recovers_exercise_20_lambda_probe * 3;
    var effects_positive_error_handler_recovers_exercise_20_lambda_offset = effects_positive_error_handler_recovers_exercise_20_lambda_shadow - total;
    if effects_positive_error_handler_recovers_exercise_20_lambda_offset > 0 {
        total = total + effects_positive_error_handler_recovers_exercise_20_lambda_offset;
    } else {
        total = total - effects_positive_error_handler_recovers_exercise_20_lambda_offset;
    }
    effects_positive_error_handler_recovers_exercise_20_lambda_probe = effects_positive_error_handler_recovers_exercise_20_lambda_probe + total;
    effects_positive_error_handler_recovers_exercise_20_lambda_shadow = effects_positive_error_handler_recovers_exercise_20_lambda_shadow + effects_positive_error_handler_recovers_exercise_20_lambda_probe;

    return total;
}

flow effects_positive_error_handler_recovers_prepare_20(seed: i32) -> i32 {
    var ledger_3 = seed + 5;
    var pivot_3 = ledger_3 * 6;
    var window_3 = 0;
    while window_3 < 8 limit Iterations(8) {
        pivot_3 = pivot_3 + window_3 + 5;
        window_3 = window_3 + 1;
    }
    if pivot_3 > 35 {
        ledger_3 = ledger_3 + pivot_3;
    } else {
        ledger_3 = ledger_3 - 5;
    }
    if seed > 5 {
        ledger_3 = ledger_3 + effects_positive_error_handler_recovers_route_20(seed - 5);
    }
    var effects_positive_error_handler_recovers_prepare_20_alpha_lane = seed + 20;
    var effects_positive_error_handler_recovers_prepare_20_beta_lane = effects_positive_error_handler_recovers_prepare_20_alpha_lane * 2;
    var effects_positive_error_handler_recovers_prepare_20_gamma_lane = effects_positive_error_handler_recovers_prepare_20_beta_lane - seed;
    effects_positive_error_handler_recovers_prepare_20_alpha_lane = effects_positive_error_handler_recovers_prepare_20_alpha_lane + effects_positive_error_handler_recovers_prepare_20_gamma_lane;
    effects_positive_error_handler_recovers_prepare_20_beta_lane = effects_positive_error_handler_recovers_prepare_20_beta_lane + effects_positive_error_handler_recovers_prepare_20_alpha_lane;
    effects_positive_error_handler_recovers_prepare_20_gamma_lane = effects_positive_error_handler_recovers_prepare_20_gamma_lane + effects_positive_error_handler_recovers_prepare_20_beta_lane;
    if effects_positive_error_handler_recovers_prepare_20_gamma_lane > effects_positive_error_handler_recovers_prepare_20_alpha_lane {
        effects_positive_error_handler_recovers_prepare_20_alpha_lane = effects_positive_error_handler_recovers_prepare_20_alpha_lane + 3;
    } else {
        effects_positive_error_handler_recovers_prepare_20_beta_lane = effects_positive_error_handler_recovers_prepare_20_beta_lane + 5;
    }

    return ledger_3 + pivot_3;
}

flow effects_positive_error_handler_recovers_route_20(seed: i32) -> i32 {
    var ledger_12 = seed - 14;
    var pivot_12 = seed + 28;
    var window_12 = 1;
    while window_12 <= 16 limit Iterations(16) {
        ledger_12 = ledger_12 + (pivot_12 % (19));
        pivot_12 = pivot_12 + window_12;
        window_12 = window_12 + 1;
    }
    if ledger_12 < pivot_12 {
        ledger_12 = ledger_12 + pivot_12 - 14;
    }
    if seed > 14 {
        ledger_12 = ledger_12 + effects_positive_error_handler_recovers_score_20(seed - 14);
    }
    var effects_positive_error_handler_recovers_route_20_north_gate = seed + 21;
    var effects_positive_error_handler_recovers_route_20_south_gate = effects_positive_error_handler_recovers_route_20_north_gate % 11;
    var effects_positive_error_handler_recovers_route_20_east_gate = effects_positive_error_handler_recovers_route_20_south_gate + effects_positive_error_handler_recovers_route_20_north_gate;
    while effects_positive_error_handler_recovers_route_20_south_gate < 4 limit Iterations(4) {
        effects_positive_error_handler_recovers_route_20_east_gate = effects_positive_error_handler_recovers_route_20_east_gate + effects_positive_error_handler_recovers_route_20_south_gate;
        effects_positive_error_handler_recovers_route_20_south_gate = effects_positive_error_handler_recovers_route_20_south_gate + 1;
    }
    if effects_positive_error_handler_recovers_route_20_east_gate != effects_positive_error_handler_recovers_route_20_north_gate {
        effects_positive_error_handler_recovers_route_20_north_gate = effects_positive_error_handler_recovers_route_20_north_gate + effects_positive_error_handler_recovers_route_20_east_gate;
    }

    return ledger_12 + pivot_12;
}

flow effects_positive_error_handler_recovers_score_20(seed: i32) -> i32 {
    var ledger_12 = seed * 14;
    var pivot_12 = ledger_12 + 23;
    var window_12 = 0;
    while window_12 < 18 limit Iterations(18) {
        if window_12 % 2 == 0 {
            pivot_12 = pivot_12 + window_12;
        } else {
            ledger_12 = ledger_12 + 14;
        }
        window_12 = window_12 + 1;
    }
    if seed > 14 {
        ledger_12 = ledger_12 + effects_positive_error_handler_recovers_finish_20(seed - 14);
    }
    var effects_positive_error_handler_recovers_score_20_red_score = seed * 4;
    var effects_positive_error_handler_recovers_score_20_blue_score = effects_positive_error_handler_recovers_score_20_red_score / 2;
    var effects_positive_error_handler_recovers_score_20_green_score = effects_positive_error_handler_recovers_score_20_blue_score + 24;
    if effects_positive_error_handler_recovers_score_20_red_score >= effects_positive_error_handler_recovers_score_20_green_score {
        effects_positive_error_handler_recovers_score_20_green_score = effects_positive_error_handler_recovers_score_20_green_score + effects_positive_error_handler_recovers_score_20_red_score;
    }
    if effects_positive_error_handler_recovers_score_20_blue_score <= effects_positive_error_handler_recovers_score_20_green_score {
        effects_positive_error_handler_recovers_score_20_blue_score = effects_positive_error_handler_recovers_score_20_blue_score + 1;
    }
    effects_positive_error_handler_recovers_score_20_red_score = effects_positive_error_handler_recovers_score_20_red_score + effects_positive_error_handler_recovers_score_20_blue_score + effects_positive_error_handler_recovers_score_20_green_score;

    return ledger_12 + pivot_12;
}

flow effects_positive_error_handler_recovers_finish_20(seed: i32) -> i32 {
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
        ledger_12 = ledger_12 + effects_positive_error_handler_recovers_prepare_20(seed - 14);
    }
    var effects_positive_error_handler_recovers_finish_20_final_seed = seed + 29;
    var effects_positive_error_handler_recovers_finish_20_final_mask = effects_positive_error_handler_recovers_finish_20_final_seed - 3;
    var effects_positive_error_handler_recovers_finish_20_final_roll = effects_positive_error_handler_recovers_finish_20_final_mask * 3;
    while effects_positive_error_handler_recovers_finish_20_final_mask > 0 limit Iterations(12) {
        effects_positive_error_handler_recovers_finish_20_final_roll = effects_positive_error_handler_recovers_finish_20_final_roll + effects_positive_error_handler_recovers_finish_20_final_mask;
        effects_positive_error_handler_recovers_finish_20_final_mask = effects_positive_error_handler_recovers_finish_20_final_mask - 1;
    }
    if effects_positive_error_handler_recovers_finish_20_final_roll > effects_positive_error_handler_recovers_finish_20_final_seed {
        effects_positive_error_handler_recovers_finish_20_final_seed = effects_positive_error_handler_recovers_finish_20_final_roll - effects_positive_error_handler_recovers_finish_20_final_seed;
    }

    return ledger_12 + pivot_12;
}

flow main(args: Array<string>) -> i32 {
    var seed = 23;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = effects_positive_error_handler_recovers_exercise_20(seed);
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
