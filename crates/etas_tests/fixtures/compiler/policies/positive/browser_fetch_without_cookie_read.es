// adjusted policy positive fixture: public etas check currently supports this effect shape
module tests.compiler.policies.positive.browser_fetch_without_cookie_read;

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

flow effects_positive_flow_value_latent_effect_exercise_21(seed: i32) -> i32 ![Web.fetch] {
    var total = effects_positive_flow_value_latent_effect_prepare_21(seed);
    total = total + effects_positive_flow_value_latent_effect_route_21(seed + 1);
    if total < 0 {
        total = 0 - total;
    }
    let effects_positive_flow_value_latent_effect_adjust_21: i32 -> i32 = (value: i32) => value + 21;
    total = effects_positive_flow_value_latent_effect_adjust_21(total);
    total = total + perform Web.fetch("docs.example.com", "/latent");
    total = total + effects_positive_flow_value_latent_effect_score_21(2);
    total = total + effects_positive_flow_value_latent_effect_finish_21(1);
    if total > 2000 {
        total = total - 2000;
    } else {
        total = total + 38;
    }
    var effects_positive_flow_value_latent_effect_exercise_21_lambda_probe = seed + 34;
    var effects_positive_flow_value_latent_effect_exercise_21_lambda_shadow = effects_positive_flow_value_latent_effect_exercise_21_lambda_probe * 3;
    var effects_positive_flow_value_latent_effect_exercise_21_lambda_offset = effects_positive_flow_value_latent_effect_exercise_21_lambda_shadow - total;
    if effects_positive_flow_value_latent_effect_exercise_21_lambda_offset > 0 {
        total = total + effects_positive_flow_value_latent_effect_exercise_21_lambda_offset;
    } else {
        total = total - effects_positive_flow_value_latent_effect_exercise_21_lambda_offset;
    }
    effects_positive_flow_value_latent_effect_exercise_21_lambda_probe = effects_positive_flow_value_latent_effect_exercise_21_lambda_probe + total;
    effects_positive_flow_value_latent_effect_exercise_21_lambda_shadow = effects_positive_flow_value_latent_effect_exercise_21_lambda_shadow + effects_positive_flow_value_latent_effect_exercise_21_lambda_probe;

    return total;
}

flow effects_positive_flow_value_latent_effect_prepare_21(seed: i32) -> i32 {
    var ledger_4 = seed - 6;
    var pivot_4 = seed + 12;
    var window_4 = 1;
    while window_4 <= 8 limit Iterations(8) {
        ledger_4 = ledger_4 + (pivot_4 % (11));
        pivot_4 = pivot_4 + window_4;
        window_4 = window_4 + 1;
    }
    if ledger_4 < pivot_4 {
        ledger_4 = ledger_4 + pivot_4 - 6;
    }
    if seed > 6 {
        ledger_4 = ledger_4 + effects_positive_flow_value_latent_effect_route_21(seed - 6);
    }
    var effects_positive_flow_value_latent_effect_prepare_21_alpha_lane = seed + 21;
    var effects_positive_flow_value_latent_effect_prepare_21_beta_lane = effects_positive_flow_value_latent_effect_prepare_21_alpha_lane * 2;
    var effects_positive_flow_value_latent_effect_prepare_21_gamma_lane = effects_positive_flow_value_latent_effect_prepare_21_beta_lane - seed;
    effects_positive_flow_value_latent_effect_prepare_21_alpha_lane = effects_positive_flow_value_latent_effect_prepare_21_alpha_lane + effects_positive_flow_value_latent_effect_prepare_21_gamma_lane;
    effects_positive_flow_value_latent_effect_prepare_21_beta_lane = effects_positive_flow_value_latent_effect_prepare_21_beta_lane + effects_positive_flow_value_latent_effect_prepare_21_alpha_lane;
    effects_positive_flow_value_latent_effect_prepare_21_gamma_lane = effects_positive_flow_value_latent_effect_prepare_21_gamma_lane + effects_positive_flow_value_latent_effect_prepare_21_beta_lane;
    if effects_positive_flow_value_latent_effect_prepare_21_gamma_lane > effects_positive_flow_value_latent_effect_prepare_21_alpha_lane {
        effects_positive_flow_value_latent_effect_prepare_21_alpha_lane = effects_positive_flow_value_latent_effect_prepare_21_alpha_lane + 3;
    } else {
        effects_positive_flow_value_latent_effect_prepare_21_beta_lane = effects_positive_flow_value_latent_effect_prepare_21_beta_lane + 5;
    }

    return ledger_4 + pivot_4;
}

flow effects_positive_flow_value_latent_effect_route_21(seed: i32) -> i32 {
    var ledger_13 = seed * 15;
    var pivot_13 = ledger_13 + 24;
    var window_13 = 0;
    while window_13 < 19 limit Iterations(19) {
        if window_13 % 2 == 0 {
            pivot_13 = pivot_13 + window_13;
        } else {
            ledger_13 = ledger_13 + 15;
        }
        window_13 = window_13 + 1;
    }
    if seed > 15 {
        ledger_13 = ledger_13 + effects_positive_flow_value_latent_effect_score_21(seed - 15);
    }
    var effects_positive_flow_value_latent_effect_route_21_north_gate = seed + 22;
    var effects_positive_flow_value_latent_effect_route_21_south_gate = effects_positive_flow_value_latent_effect_route_21_north_gate % 5;
    var effects_positive_flow_value_latent_effect_route_21_east_gate = effects_positive_flow_value_latent_effect_route_21_south_gate + effects_positive_flow_value_latent_effect_route_21_north_gate;
    while effects_positive_flow_value_latent_effect_route_21_south_gate < 4 limit Iterations(4) {
        effects_positive_flow_value_latent_effect_route_21_east_gate = effects_positive_flow_value_latent_effect_route_21_east_gate + effects_positive_flow_value_latent_effect_route_21_south_gate;
        effects_positive_flow_value_latent_effect_route_21_south_gate = effects_positive_flow_value_latent_effect_route_21_south_gate + 1;
    }
    if effects_positive_flow_value_latent_effect_route_21_east_gate != effects_positive_flow_value_latent_effect_route_21_north_gate {
        effects_positive_flow_value_latent_effect_route_21_north_gate = effects_positive_flow_value_latent_effect_route_21_north_gate + effects_positive_flow_value_latent_effect_route_21_east_gate;
    }

    return ledger_13 + pivot_13;
}

flow effects_positive_flow_value_latent_effect_score_21(seed: i32) -> i32 {
    var ledger_13 = seed + 45;
    var pivot_13 = ledger_13 / 15;
    var window_13 = 15;
    while window_13 > 0 limit Iterations(16) {
        pivot_13 = pivot_13 + window_13;
        ledger_13 = ledger_13 + pivot_13;
        window_13 = window_13 - 1;
    }
    if ledger_13 != pivot_13 {
        ledger_13 = ledger_13 + 15;
    }
    if seed > 15 {
        ledger_13 = ledger_13 + effects_positive_flow_value_latent_effect_finish_21(seed - 15);
    }
    var effects_positive_flow_value_latent_effect_score_21_red_score = seed * 5;
    var effects_positive_flow_value_latent_effect_score_21_blue_score = effects_positive_flow_value_latent_effect_score_21_red_score / 3;
    var effects_positive_flow_value_latent_effect_score_21_green_score = effects_positive_flow_value_latent_effect_score_21_blue_score + 25;
    if effects_positive_flow_value_latent_effect_score_21_red_score >= effects_positive_flow_value_latent_effect_score_21_green_score {
        effects_positive_flow_value_latent_effect_score_21_green_score = effects_positive_flow_value_latent_effect_score_21_green_score + effects_positive_flow_value_latent_effect_score_21_red_score;
    }
    if effects_positive_flow_value_latent_effect_score_21_blue_score <= effects_positive_flow_value_latent_effect_score_21_green_score {
        effects_positive_flow_value_latent_effect_score_21_blue_score = effects_positive_flow_value_latent_effect_score_21_blue_score + 1;
    }
    effects_positive_flow_value_latent_effect_score_21_red_score = effects_positive_flow_value_latent_effect_score_21_red_score + effects_positive_flow_value_latent_effect_score_21_blue_score + effects_positive_flow_value_latent_effect_score_21_green_score;

    return ledger_13 + pivot_13;
}

flow effects_positive_flow_value_latent_effect_finish_21(seed: i32) -> i32 {
    var ledger_13 = seed + 15;
    var pivot_13 = ledger_13 * 16;
    var window_13 = 0;
    while window_13 < 18 limit Iterations(18) {
        pivot_13 = pivot_13 + window_13 + 15;
        window_13 = window_13 + 1;
    }
    if pivot_13 > 105 {
        ledger_13 = ledger_13 + pivot_13;
    } else {
        ledger_13 = ledger_13 - 15;
    }
    if seed > 15 {
        ledger_13 = ledger_13 + effects_positive_flow_value_latent_effect_prepare_21(seed - 15);
    }
    var effects_positive_flow_value_latent_effect_finish_21_final_seed = seed + 30;
    var effects_positive_flow_value_latent_effect_finish_21_final_mask = effects_positive_flow_value_latent_effect_finish_21_final_seed - 4;
    var effects_positive_flow_value_latent_effect_finish_21_final_roll = effects_positive_flow_value_latent_effect_finish_21_final_mask * 4;
    while effects_positive_flow_value_latent_effect_finish_21_final_mask > 0 limit Iterations(12) {
        effects_positive_flow_value_latent_effect_finish_21_final_roll = effects_positive_flow_value_latent_effect_finish_21_final_roll + effects_positive_flow_value_latent_effect_finish_21_final_mask;
        effects_positive_flow_value_latent_effect_finish_21_final_mask = effects_positive_flow_value_latent_effect_finish_21_final_mask - 1;
    }
    if effects_positive_flow_value_latent_effect_finish_21_final_roll > effects_positive_flow_value_latent_effect_finish_21_final_seed {
        effects_positive_flow_value_latent_effect_finish_21_final_seed = effects_positive_flow_value_latent_effect_finish_21_final_roll - effects_positive_flow_value_latent_effect_finish_21_final_seed;
    }

    return ledger_13 + pivot_13;
}

flow main(args: Array<string>) -> i32 {
    var seed = 24;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = effects_positive_flow_value_latent_effect_exercise_21(seed);
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
