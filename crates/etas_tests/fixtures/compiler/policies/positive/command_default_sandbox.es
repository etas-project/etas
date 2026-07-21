// adjusted policy positive fixture: public etas check currently supports this effect shape
module tests.compiler.policies.positive.command_default_sandbox;

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

agent FixtureAgent(input: string) -> i32 {}

flow effects_positive_model_and_trace_union_exercise_28(seed: i32) -> i32 ![Trace.emit] {
    var total = effects_positive_model_and_trace_union_prepare_28(seed);
    total = total + effects_positive_model_and_trace_union_route_28(seed + 1);
    if total < 0 {
        total = 0 - total;
    }
    let effects_positive_model_and_trace_union_adjust_28: i32 -> i32 = (value: i32) => value + 28;
    total = effects_positive_model_and_trace_union_adjust_28(total);
    perform Trace.emit("model.start", "effect");
    total = total + FixtureAgent.run("summarize effects");
    perform Trace.emit("model.end", "effect");
    total = total + effects_positive_model_and_trace_union_score_28(2);
    total = total + effects_positive_model_and_trace_union_finish_28(1);
    if total > 2000 {
        total = total - 2000;
    } else {
        total = total + 45;
    }
    var effects_positive_model_and_trace_union_exercise_28_lambda_probe = seed + 41;
    var effects_positive_model_and_trace_union_exercise_28_lambda_shadow = effects_positive_model_and_trace_union_exercise_28_lambda_probe * 3;
    var effects_positive_model_and_trace_union_exercise_28_lambda_offset = effects_positive_model_and_trace_union_exercise_28_lambda_shadow - total;
    if effects_positive_model_and_trace_union_exercise_28_lambda_offset > 0 {
        total = total + effects_positive_model_and_trace_union_exercise_28_lambda_offset;
    } else {
        total = total - effects_positive_model_and_trace_union_exercise_28_lambda_offset;
    }
    effects_positive_model_and_trace_union_exercise_28_lambda_probe = effects_positive_model_and_trace_union_exercise_28_lambda_probe + total;
    effects_positive_model_and_trace_union_exercise_28_lambda_shadow = effects_positive_model_and_trace_union_exercise_28_lambda_shadow + effects_positive_model_and_trace_union_exercise_28_lambda_probe;

    return total;
}

flow effects_positive_model_and_trace_union_prepare_28(seed: i32) -> i32 {
    var ledger_2 = seed + 4;
    var pivot_2 = ledger_2 * 5;
    var window_2 = 0;
    while window_2 < 7 limit Iterations(7) {
        pivot_2 = pivot_2 + window_2 + 4;
        window_2 = window_2 + 1;
    }
    if pivot_2 > 28 {
        ledger_2 = ledger_2 + pivot_2;
    } else {
        ledger_2 = ledger_2 - 4;
    }
    if seed > 4 {
        ledger_2 = ledger_2 + effects_positive_model_and_trace_union_route_28(seed - 4);
    }
    var effects_positive_model_and_trace_union_prepare_28_alpha_lane = seed + 28;
    var effects_positive_model_and_trace_union_prepare_28_beta_lane = effects_positive_model_and_trace_union_prepare_28_alpha_lane * 2;
    var effects_positive_model_and_trace_union_prepare_28_gamma_lane = effects_positive_model_and_trace_union_prepare_28_beta_lane - seed;
    effects_positive_model_and_trace_union_prepare_28_alpha_lane = effects_positive_model_and_trace_union_prepare_28_alpha_lane + effects_positive_model_and_trace_union_prepare_28_gamma_lane;
    effects_positive_model_and_trace_union_prepare_28_beta_lane = effects_positive_model_and_trace_union_prepare_28_beta_lane + effects_positive_model_and_trace_union_prepare_28_alpha_lane;
    effects_positive_model_and_trace_union_prepare_28_gamma_lane = effects_positive_model_and_trace_union_prepare_28_gamma_lane + effects_positive_model_and_trace_union_prepare_28_beta_lane;
    if effects_positive_model_and_trace_union_prepare_28_gamma_lane > effects_positive_model_and_trace_union_prepare_28_alpha_lane {
        effects_positive_model_and_trace_union_prepare_28_alpha_lane = effects_positive_model_and_trace_union_prepare_28_alpha_lane + 3;
    } else {
        effects_positive_model_and_trace_union_prepare_28_beta_lane = effects_positive_model_and_trace_union_prepare_28_beta_lane + 5;
    }

    return ledger_2 + pivot_2;
}

flow effects_positive_model_and_trace_union_route_28(seed: i32) -> i32 {
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
        ledger_9 = ledger_9 + effects_positive_model_and_trace_union_score_28(seed - 11);
    }
    var effects_positive_model_and_trace_union_route_28_north_gate = seed + 29;
    var effects_positive_model_and_trace_union_route_28_south_gate = effects_positive_model_and_trace_union_route_28_north_gate % 5;
    var effects_positive_model_and_trace_union_route_28_east_gate = effects_positive_model_and_trace_union_route_28_south_gate + effects_positive_model_and_trace_union_route_28_north_gate;
    while effects_positive_model_and_trace_union_route_28_south_gate < 4 limit Iterations(4) {
        effects_positive_model_and_trace_union_route_28_east_gate = effects_positive_model_and_trace_union_route_28_east_gate + effects_positive_model_and_trace_union_route_28_south_gate;
        effects_positive_model_and_trace_union_route_28_south_gate = effects_positive_model_and_trace_union_route_28_south_gate + 1;
    }
    if effects_positive_model_and_trace_union_route_28_east_gate != effects_positive_model_and_trace_union_route_28_north_gate {
        effects_positive_model_and_trace_union_route_28_north_gate = effects_positive_model_and_trace_union_route_28_north_gate + effects_positive_model_and_trace_union_route_28_east_gate;
    }

    return ledger_9 + pivot_9;
}

flow effects_positive_model_and_trace_union_score_28(seed: i32) -> i32 {
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
        ledger_7 = ledger_7 + effects_positive_model_and_trace_union_finish_28(seed - 9);
    }
    var effects_positive_model_and_trace_union_score_28_red_score = seed * 3;
    var effects_positive_model_and_trace_union_score_28_blue_score = effects_positive_model_and_trace_union_score_28_red_score / 5;
    var effects_positive_model_and_trace_union_score_28_green_score = effects_positive_model_and_trace_union_score_28_blue_score + 32;
    if effects_positive_model_and_trace_union_score_28_red_score >= effects_positive_model_and_trace_union_score_28_green_score {
        effects_positive_model_and_trace_union_score_28_green_score = effects_positive_model_and_trace_union_score_28_green_score + effects_positive_model_and_trace_union_score_28_red_score;
    }
    if effects_positive_model_and_trace_union_score_28_blue_score <= effects_positive_model_and_trace_union_score_28_green_score {
        effects_positive_model_and_trace_union_score_28_blue_score = effects_positive_model_and_trace_union_score_28_blue_score + 1;
    }
    effects_positive_model_and_trace_union_score_28_red_score = effects_positive_model_and_trace_union_score_28_red_score + effects_positive_model_and_trace_union_score_28_blue_score + effects_positive_model_and_trace_union_score_28_green_score;

    return ledger_7 + pivot_7;
}

flow effects_positive_model_and_trace_union_finish_28(seed: i32) -> i32 {
    var ledger_20 = seed + 66;
    var pivot_20 = ledger_20 / 22;
    var window_20 = 22;
    while window_20 > 0 limit Iterations(23) {
        pivot_20 = pivot_20 + window_20;
        ledger_20 = ledger_20 + pivot_20;
        window_20 = window_20 - 1;
    }
    if ledger_20 != pivot_20 {
        ledger_20 = ledger_20 + 22;
    }
    if seed > 22 {
        ledger_20 = ledger_20 + effects_positive_model_and_trace_union_prepare_28(seed - 22);
    }
    var effects_positive_model_and_trace_union_finish_28_final_seed = seed + 37;
    var effects_positive_model_and_trace_union_finish_28_final_mask = effects_positive_model_and_trace_union_finish_28_final_seed - 5;
    var effects_positive_model_and_trace_union_finish_28_final_roll = effects_positive_model_and_trace_union_finish_28_final_mask * 3;
    while effects_positive_model_and_trace_union_finish_28_final_mask > 0 limit Iterations(12) {
        effects_positive_model_and_trace_union_finish_28_final_roll = effects_positive_model_and_trace_union_finish_28_final_roll + effects_positive_model_and_trace_union_finish_28_final_mask;
        effects_positive_model_and_trace_union_finish_28_final_mask = effects_positive_model_and_trace_union_finish_28_final_mask - 1;
    }
    if effects_positive_model_and_trace_union_finish_28_final_roll > effects_positive_model_and_trace_union_finish_28_final_seed {
        effects_positive_model_and_trace_union_finish_28_final_seed = effects_positive_model_and_trace_union_finish_28_final_roll - effects_positive_model_and_trace_union_finish_28_final_seed;
    }

    return ledger_20 + pivot_20;
}

flow main(args: Array<string>) -> i32 {
    var seed = 31;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = effects_positive_model_and_trace_union_exercise_28(seed);
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
