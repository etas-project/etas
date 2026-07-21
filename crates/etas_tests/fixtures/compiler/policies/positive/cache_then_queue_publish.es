// adjusted policy positive fixture: public etas check currently supports this effect shape
module tests.compiler.policies.positive.cache_then_queue_publish;

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

flow effects_positive_impl_declares_effect_actions_exercise_25(seed: i32) -> i32 ![Audit.write] {
    var total = effects_positive_impl_declares_effect_actions_prepare_25(seed);
    total = total + effects_positive_impl_declares_effect_actions_route_25(seed + 1);
    if total < 0 {
        total = 0 - total;
    }
    let effects_positive_impl_declares_effect_actions_adjust_25: i32 -> i32 = (value: i32) => value + 25;
    total = effects_positive_impl_declares_effect_actions_adjust_25(total);
    perform Audit.write("impl action " + "declared");
    total = total + 4;
    total = total + effects_positive_impl_declares_effect_actions_score_25(2);
    total = total + effects_positive_impl_declares_effect_actions_finish_25(1);
    if total > 2000 {
        total = total - 2000;
    } else {
        total = total + 42;
    }
    var effects_positive_impl_declares_effect_actions_exercise_25_lambda_probe = seed + 38;
    var effects_positive_impl_declares_effect_actions_exercise_25_lambda_shadow = effects_positive_impl_declares_effect_actions_exercise_25_lambda_probe * 3;
    var effects_positive_impl_declares_effect_actions_exercise_25_lambda_offset = effects_positive_impl_declares_effect_actions_exercise_25_lambda_shadow - total;
    if effects_positive_impl_declares_effect_actions_exercise_25_lambda_offset > 0 {
        total = total + effects_positive_impl_declares_effect_actions_exercise_25_lambda_offset;
    } else {
        total = total - effects_positive_impl_declares_effect_actions_exercise_25_lambda_offset;
    }
    effects_positive_impl_declares_effect_actions_exercise_25_lambda_probe = effects_positive_impl_declares_effect_actions_exercise_25_lambda_probe + total;
    effects_positive_impl_declares_effect_actions_exercise_25_lambda_shadow = effects_positive_impl_declares_effect_actions_exercise_25_lambda_shadow + effects_positive_impl_declares_effect_actions_exercise_25_lambda_probe;

    return total;
}

flow effects_positive_impl_declares_effect_actions_prepare_25(seed: i32) -> i32 {
    var ledger_8 = seed - 10;
    var pivot_8 = seed + 20;
    var window_8 = 1;
    while window_8 <= 12 limit Iterations(12) {
        ledger_8 = ledger_8 + (pivot_8 % (15));
        pivot_8 = pivot_8 + window_8;
        window_8 = window_8 + 1;
    }
    if ledger_8 < pivot_8 {
        ledger_8 = ledger_8 + pivot_8 - 10;
    }
    if seed > 10 {
        ledger_8 = ledger_8 + effects_positive_impl_declares_effect_actions_route_25(seed - 10);
    }
    var effects_positive_impl_declares_effect_actions_prepare_25_alpha_lane = seed + 25;
    var effects_positive_impl_declares_effect_actions_prepare_25_beta_lane = effects_positive_impl_declares_effect_actions_prepare_25_alpha_lane * 2;
    var effects_positive_impl_declares_effect_actions_prepare_25_gamma_lane = effects_positive_impl_declares_effect_actions_prepare_25_beta_lane - seed;
    effects_positive_impl_declares_effect_actions_prepare_25_alpha_lane = effects_positive_impl_declares_effect_actions_prepare_25_alpha_lane + effects_positive_impl_declares_effect_actions_prepare_25_gamma_lane;
    effects_positive_impl_declares_effect_actions_prepare_25_beta_lane = effects_positive_impl_declares_effect_actions_prepare_25_beta_lane + effects_positive_impl_declares_effect_actions_prepare_25_alpha_lane;
    effects_positive_impl_declares_effect_actions_prepare_25_gamma_lane = effects_positive_impl_declares_effect_actions_prepare_25_gamma_lane + effects_positive_impl_declares_effect_actions_prepare_25_beta_lane;
    if effects_positive_impl_declares_effect_actions_prepare_25_gamma_lane > effects_positive_impl_declares_effect_actions_prepare_25_alpha_lane {
        effects_positive_impl_declares_effect_actions_prepare_25_alpha_lane = effects_positive_impl_declares_effect_actions_prepare_25_alpha_lane + 3;
    } else {
        effects_positive_impl_declares_effect_actions_prepare_25_beta_lane = effects_positive_impl_declares_effect_actions_prepare_25_beta_lane + 5;
    }

    return ledger_8 + pivot_8;
}

flow effects_positive_impl_declares_effect_actions_route_25(seed: i32) -> i32 {
    var ledger_6 = seed * 8;
    var pivot_6 = ledger_6 + 17;
    var window_6 = 0;
    while window_6 < 12 limit Iterations(12) {
        if window_6 % 2 == 0 {
            pivot_6 = pivot_6 + window_6;
        } else {
            ledger_6 = ledger_6 + 8;
        }
        window_6 = window_6 + 1;
    }
    if seed > 8 {
        ledger_6 = ledger_6 + effects_positive_impl_declares_effect_actions_score_25(seed - 8);
    }
    var effects_positive_impl_declares_effect_actions_route_25_north_gate = seed + 26;
    var effects_positive_impl_declares_effect_actions_route_25_south_gate = effects_positive_impl_declares_effect_actions_route_25_north_gate % 9;
    var effects_positive_impl_declares_effect_actions_route_25_east_gate = effects_positive_impl_declares_effect_actions_route_25_south_gate + effects_positive_impl_declares_effect_actions_route_25_north_gate;
    while effects_positive_impl_declares_effect_actions_route_25_south_gate < 4 limit Iterations(4) {
        effects_positive_impl_declares_effect_actions_route_25_east_gate = effects_positive_impl_declares_effect_actions_route_25_east_gate + effects_positive_impl_declares_effect_actions_route_25_south_gate;
        effects_positive_impl_declares_effect_actions_route_25_south_gate = effects_positive_impl_declares_effect_actions_route_25_south_gate + 1;
    }
    if effects_positive_impl_declares_effect_actions_route_25_east_gate != effects_positive_impl_declares_effect_actions_route_25_north_gate {
        effects_positive_impl_declares_effect_actions_route_25_north_gate = effects_positive_impl_declares_effect_actions_route_25_north_gate + effects_positive_impl_declares_effect_actions_route_25_east_gate;
    }

    return ledger_6 + pivot_6;
}

flow effects_positive_impl_declares_effect_actions_score_25(seed: i32) -> i32 {
    var ledger_17 = seed + 57;
    var pivot_17 = ledger_17 / 19;
    var window_17 = 19;
    while window_17 > 0 limit Iterations(20) {
        pivot_17 = pivot_17 + window_17;
        ledger_17 = ledger_17 + pivot_17;
        window_17 = window_17 - 1;
    }
    if ledger_17 != pivot_17 {
        ledger_17 = ledger_17 + 19;
    }
    if seed > 19 {
        ledger_17 = ledger_17 + effects_positive_impl_declares_effect_actions_finish_25(seed - 19);
    }
    var effects_positive_impl_declares_effect_actions_score_25_red_score = seed * 9;
    var effects_positive_impl_declares_effect_actions_score_25_blue_score = effects_positive_impl_declares_effect_actions_score_25_red_score / 2;
    var effects_positive_impl_declares_effect_actions_score_25_green_score = effects_positive_impl_declares_effect_actions_score_25_blue_score + 29;
    if effects_positive_impl_declares_effect_actions_score_25_red_score >= effects_positive_impl_declares_effect_actions_score_25_green_score {
        effects_positive_impl_declares_effect_actions_score_25_green_score = effects_positive_impl_declares_effect_actions_score_25_green_score + effects_positive_impl_declares_effect_actions_score_25_red_score;
    }
    if effects_positive_impl_declares_effect_actions_score_25_blue_score <= effects_positive_impl_declares_effect_actions_score_25_green_score {
        effects_positive_impl_declares_effect_actions_score_25_blue_score = effects_positive_impl_declares_effect_actions_score_25_blue_score + 1;
    }
    effects_positive_impl_declares_effect_actions_score_25_red_score = effects_positive_impl_declares_effect_actions_score_25_red_score + effects_positive_impl_declares_effect_actions_score_25_blue_score + effects_positive_impl_declares_effect_actions_score_25_green_score;

    return ledger_17 + pivot_17;
}

flow effects_positive_impl_declares_effect_actions_finish_25(seed: i32) -> i32 {
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
        ledger_17 = ledger_17 + effects_positive_impl_declares_effect_actions_prepare_25(seed - 19);
    }
    var effects_positive_impl_declares_effect_actions_finish_25_final_seed = seed + 34;
    var effects_positive_impl_declares_effect_actions_finish_25_final_mask = effects_positive_impl_declares_effect_actions_finish_25_final_seed - 2;
    var effects_positive_impl_declares_effect_actions_finish_25_final_roll = effects_positive_impl_declares_effect_actions_finish_25_final_mask * 4;
    while effects_positive_impl_declares_effect_actions_finish_25_final_mask > 0 limit Iterations(12) {
        effects_positive_impl_declares_effect_actions_finish_25_final_roll = effects_positive_impl_declares_effect_actions_finish_25_final_roll + effects_positive_impl_declares_effect_actions_finish_25_final_mask;
        effects_positive_impl_declares_effect_actions_finish_25_final_mask = effects_positive_impl_declares_effect_actions_finish_25_final_mask - 1;
    }
    if effects_positive_impl_declares_effect_actions_finish_25_final_roll > effects_positive_impl_declares_effect_actions_finish_25_final_seed {
        effects_positive_impl_declares_effect_actions_finish_25_final_seed = effects_positive_impl_declares_effect_actions_finish_25_final_roll - effects_positive_impl_declares_effect_actions_finish_25_final_seed;
    }

    return ledger_17 + pivot_17;
}

flow main(args: Array<string>) -> i32 {
    var seed = 28;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = effects_positive_impl_declares_effect_actions_exercise_25(seed);
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
