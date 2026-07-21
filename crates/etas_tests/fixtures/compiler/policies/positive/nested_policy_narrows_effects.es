// adjusted policy positive fixture: public etas check currently supports this effect shape
module tests.compiler.policies.positive.nested_policy_narrows_effects;

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

flow effects_positive_parameterized_workspace_effects_exercise_29(seed: i32) -> i32 ![Workspace.read, Workspace.write] {
    var total = effects_positive_parameterized_workspace_effects_prepare_29(seed);
    total = total + effects_positive_parameterized_workspace_effects_route_29(seed + 1);
    if total < 0 {
        total = 0 - total;
    }
    let effects_positive_parameterized_workspace_effects_adjust_29: i32 -> i32 = (value: i32) => value + 29;
    total = effects_positive_parameterized_workspace_effects_adjust_29(total);
    let template = perform Workspace.read("templates/**", "templates/report.md");
    perform Workspace.write("reports/**", "reports/out.md", "body");
    total = total + template;
    total = total + effects_positive_parameterized_workspace_effects_score_29(2);
    total = total + effects_positive_parameterized_workspace_effects_finish_29(1);
    if total > 2000 {
        total = total - 2000;
    } else {
        total = total + 46;
    }
    var effects_positive_parameterized_workspace_effects_exercise_29_lambda_probe = seed + 42;
    var effects_positive_parameterized_workspace_effects_exercise_29_lambda_shadow = effects_positive_parameterized_workspace_effects_exercise_29_lambda_probe * 3;
    var effects_positive_parameterized_workspace_effects_exercise_29_lambda_offset = effects_positive_parameterized_workspace_effects_exercise_29_lambda_shadow - total;
    if effects_positive_parameterized_workspace_effects_exercise_29_lambda_offset > 0 {
        total = total + effects_positive_parameterized_workspace_effects_exercise_29_lambda_offset;
    } else {
        total = total - effects_positive_parameterized_workspace_effects_exercise_29_lambda_offset;
    }
    effects_positive_parameterized_workspace_effects_exercise_29_lambda_probe = effects_positive_parameterized_workspace_effects_exercise_29_lambda_probe + total;
    effects_positive_parameterized_workspace_effects_exercise_29_lambda_shadow = effects_positive_parameterized_workspace_effects_exercise_29_lambda_shadow + effects_positive_parameterized_workspace_effects_exercise_29_lambda_probe;

    return total;
}

flow effects_positive_parameterized_workspace_effects_prepare_29(seed: i32) -> i32 {
    var ledger_3 = seed - 5;
    var pivot_3 = seed + 10;
    var window_3 = 1;
    while window_3 <= 7 limit Iterations(7) {
        ledger_3 = ledger_3 + (pivot_3 % (10));
        pivot_3 = pivot_3 + window_3;
        window_3 = window_3 + 1;
    }
    if ledger_3 < pivot_3 {
        ledger_3 = ledger_3 + pivot_3 - 5;
    }
    if seed > 5 {
        ledger_3 = ledger_3 + effects_positive_parameterized_workspace_effects_route_29(seed - 5);
    }
    var effects_positive_parameterized_workspace_effects_prepare_29_alpha_lane = seed + 29;
    var effects_positive_parameterized_workspace_effects_prepare_29_beta_lane = effects_positive_parameterized_workspace_effects_prepare_29_alpha_lane * 2;
    var effects_positive_parameterized_workspace_effects_prepare_29_gamma_lane = effects_positive_parameterized_workspace_effects_prepare_29_beta_lane - seed;
    effects_positive_parameterized_workspace_effects_prepare_29_alpha_lane = effects_positive_parameterized_workspace_effects_prepare_29_alpha_lane + effects_positive_parameterized_workspace_effects_prepare_29_gamma_lane;
    effects_positive_parameterized_workspace_effects_prepare_29_beta_lane = effects_positive_parameterized_workspace_effects_prepare_29_beta_lane + effects_positive_parameterized_workspace_effects_prepare_29_alpha_lane;
    effects_positive_parameterized_workspace_effects_prepare_29_gamma_lane = effects_positive_parameterized_workspace_effects_prepare_29_gamma_lane + effects_positive_parameterized_workspace_effects_prepare_29_beta_lane;
    if effects_positive_parameterized_workspace_effects_prepare_29_gamma_lane > effects_positive_parameterized_workspace_effects_prepare_29_alpha_lane {
        effects_positive_parameterized_workspace_effects_prepare_29_alpha_lane = effects_positive_parameterized_workspace_effects_prepare_29_alpha_lane + 3;
    } else {
        effects_positive_parameterized_workspace_effects_prepare_29_beta_lane = effects_positive_parameterized_workspace_effects_prepare_29_beta_lane + 5;
    }

    return ledger_3 + pivot_3;
}

flow effects_positive_parameterized_workspace_effects_route_29(seed: i32) -> i32 {
    var ledger_10 = seed * 12;
    var pivot_10 = ledger_10 + 21;
    var window_10 = 0;
    while window_10 < 16 limit Iterations(16) {
        if window_10 % 2 == 0 {
            pivot_10 = pivot_10 + window_10;
        } else {
            ledger_10 = ledger_10 + 12;
        }
        window_10 = window_10 + 1;
    }
    if seed > 12 {
        ledger_10 = ledger_10 + effects_positive_parameterized_workspace_effects_score_29(seed - 12);
    }
    var effects_positive_parameterized_workspace_effects_route_29_north_gate = seed + 30;
    var effects_positive_parameterized_workspace_effects_route_29_south_gate = effects_positive_parameterized_workspace_effects_route_29_north_gate % 6;
    var effects_positive_parameterized_workspace_effects_route_29_east_gate = effects_positive_parameterized_workspace_effects_route_29_south_gate + effects_positive_parameterized_workspace_effects_route_29_north_gate;
    while effects_positive_parameterized_workspace_effects_route_29_south_gate < 4 limit Iterations(4) {
        effects_positive_parameterized_workspace_effects_route_29_east_gate = effects_positive_parameterized_workspace_effects_route_29_east_gate + effects_positive_parameterized_workspace_effects_route_29_south_gate;
        effects_positive_parameterized_workspace_effects_route_29_south_gate = effects_positive_parameterized_workspace_effects_route_29_south_gate + 1;
    }
    if effects_positive_parameterized_workspace_effects_route_29_east_gate != effects_positive_parameterized_workspace_effects_route_29_north_gate {
        effects_positive_parameterized_workspace_effects_route_29_north_gate = effects_positive_parameterized_workspace_effects_route_29_north_gate + effects_positive_parameterized_workspace_effects_route_29_east_gate;
    }

    return ledger_10 + pivot_10;
}

flow effects_positive_parameterized_workspace_effects_score_29(seed: i32) -> i32 {
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
        ledger_8 = ledger_8 + effects_positive_parameterized_workspace_effects_finish_29(seed - 10);
    }
    var effects_positive_parameterized_workspace_effects_score_29_red_score = seed * 4;
    var effects_positive_parameterized_workspace_effects_score_29_blue_score = effects_positive_parameterized_workspace_effects_score_29_red_score / 6;
    var effects_positive_parameterized_workspace_effects_score_29_green_score = effects_positive_parameterized_workspace_effects_score_29_blue_score + 33;
    if effects_positive_parameterized_workspace_effects_score_29_red_score >= effects_positive_parameterized_workspace_effects_score_29_green_score {
        effects_positive_parameterized_workspace_effects_score_29_green_score = effects_positive_parameterized_workspace_effects_score_29_green_score + effects_positive_parameterized_workspace_effects_score_29_red_score;
    }
    if effects_positive_parameterized_workspace_effects_score_29_blue_score <= effects_positive_parameterized_workspace_effects_score_29_green_score {
        effects_positive_parameterized_workspace_effects_score_29_blue_score = effects_positive_parameterized_workspace_effects_score_29_blue_score + 1;
    }
    effects_positive_parameterized_workspace_effects_score_29_red_score = effects_positive_parameterized_workspace_effects_score_29_red_score + effects_positive_parameterized_workspace_effects_score_29_blue_score + effects_positive_parameterized_workspace_effects_score_29_green_score;

    return ledger_8 + pivot_8;
}

flow effects_positive_parameterized_workspace_effects_finish_29(seed: i32) -> i32 {
    var ledger_21 = seed + 23;
    var pivot_21 = ledger_21 * 24;
    var window_21 = 0;
    while window_21 < 26 limit Iterations(26) {
        pivot_21 = pivot_21 + window_21 + 23;
        window_21 = window_21 + 1;
    }
    if pivot_21 > 161 {
        ledger_21 = ledger_21 + pivot_21;
    } else {
        ledger_21 = ledger_21 - 23;
    }
    if seed > 23 {
        ledger_21 = ledger_21 + effects_positive_parameterized_workspace_effects_prepare_29(seed - 23);
    }
    var effects_positive_parameterized_workspace_effects_finish_29_final_seed = seed + 38;
    var effects_positive_parameterized_workspace_effects_finish_29_final_mask = effects_positive_parameterized_workspace_effects_finish_29_final_seed - 6;
    var effects_positive_parameterized_workspace_effects_finish_29_final_roll = effects_positive_parameterized_workspace_effects_finish_29_final_mask * 4;
    while effects_positive_parameterized_workspace_effects_finish_29_final_mask > 0 limit Iterations(12) {
        effects_positive_parameterized_workspace_effects_finish_29_final_roll = effects_positive_parameterized_workspace_effects_finish_29_final_roll + effects_positive_parameterized_workspace_effects_finish_29_final_mask;
        effects_positive_parameterized_workspace_effects_finish_29_final_mask = effects_positive_parameterized_workspace_effects_finish_29_final_mask - 1;
    }
    if effects_positive_parameterized_workspace_effects_finish_29_final_roll > effects_positive_parameterized_workspace_effects_finish_29_final_seed {
        effects_positive_parameterized_workspace_effects_finish_29_final_seed = effects_positive_parameterized_workspace_effects_finish_29_final_roll - effects_positive_parameterized_workspace_effects_finish_29_final_seed;
    }

    return ledger_21 + pivot_21;
}

flow main(args: Array<string>) -> i32 {
    var seed = 32;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = effects_positive_parameterized_workspace_effects_exercise_29(seed);
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
