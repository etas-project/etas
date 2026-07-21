// adjusted policy positive fixture: public etas check currently supports this effect shape
module tests.compiler.policies.positive.calendar_invite_requires_approval;

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

flow effects_positive_loop_limit_network_fetch_exercise_26(seed: i32) -> i32 ![Web.fetch] {
    var total = effects_positive_loop_limit_network_fetch_prepare_26(seed);
    total = total + effects_positive_loop_limit_network_fetch_route_26(seed + 1);
    if total < 0 {
        total = 0 - total;
    }
    let effects_positive_loop_limit_network_fetch_adjust_26: i32 -> i32 = (value: i32) => value + 26;
    total = effects_positive_loop_limit_network_fetch_adjust_26(total);
    var cursor = 0;
    while cursor < 3 limit Iterations(3) {
        total = total + perform Web.fetch("docs.example.com", "/page");
        cursor = cursor + 1;
    }
    total = total + effects_positive_loop_limit_network_fetch_score_26(2);
    total = total + effects_positive_loop_limit_network_fetch_finish_26(1);
    if total > 2000 {
        total = total - 2000;
    } else {
        total = total + 43;
    }
    var effects_positive_loop_limit_network_fetch_exercise_26_lambda_probe = seed + 39;
    var effects_positive_loop_limit_network_fetch_exercise_26_lambda_shadow = effects_positive_loop_limit_network_fetch_exercise_26_lambda_probe * 3;
    var effects_positive_loop_limit_network_fetch_exercise_26_lambda_offset = effects_positive_loop_limit_network_fetch_exercise_26_lambda_shadow - total;
    if effects_positive_loop_limit_network_fetch_exercise_26_lambda_offset > 0 {
        total = total + effects_positive_loop_limit_network_fetch_exercise_26_lambda_offset;
    } else {
        total = total - effects_positive_loop_limit_network_fetch_exercise_26_lambda_offset;
    }
    effects_positive_loop_limit_network_fetch_exercise_26_lambda_probe = effects_positive_loop_limit_network_fetch_exercise_26_lambda_probe + total;
    effects_positive_loop_limit_network_fetch_exercise_26_lambda_shadow = effects_positive_loop_limit_network_fetch_exercise_26_lambda_shadow + effects_positive_loop_limit_network_fetch_exercise_26_lambda_probe;

    return total;
}

flow effects_positive_loop_limit_network_fetch_prepare_26(seed: i32) -> i32 {
    var ledger_9 = seed * 11;
    var pivot_9 = ledger_9 + 20;
    var window_9 = 0;
    while window_9 < 15 limit Iterations(15) {
        if window_9 % 2 == 0 {
            pivot_9 = pivot_9 + window_9;
        } else {
            ledger_9 = ledger_9 + 11;
        }
        window_9 = window_9 + 1;
    }
    if seed > 11 {
        ledger_9 = ledger_9 + effects_positive_loop_limit_network_fetch_route_26(seed - 11);
    }
    var effects_positive_loop_limit_network_fetch_prepare_26_alpha_lane = seed + 26;
    var effects_positive_loop_limit_network_fetch_prepare_26_beta_lane = effects_positive_loop_limit_network_fetch_prepare_26_alpha_lane * 2;
    var effects_positive_loop_limit_network_fetch_prepare_26_gamma_lane = effects_positive_loop_limit_network_fetch_prepare_26_beta_lane - seed;
    effects_positive_loop_limit_network_fetch_prepare_26_alpha_lane = effects_positive_loop_limit_network_fetch_prepare_26_alpha_lane + effects_positive_loop_limit_network_fetch_prepare_26_gamma_lane;
    effects_positive_loop_limit_network_fetch_prepare_26_beta_lane = effects_positive_loop_limit_network_fetch_prepare_26_beta_lane + effects_positive_loop_limit_network_fetch_prepare_26_alpha_lane;
    effects_positive_loop_limit_network_fetch_prepare_26_gamma_lane = effects_positive_loop_limit_network_fetch_prepare_26_gamma_lane + effects_positive_loop_limit_network_fetch_prepare_26_beta_lane;
    if effects_positive_loop_limit_network_fetch_prepare_26_gamma_lane > effects_positive_loop_limit_network_fetch_prepare_26_alpha_lane {
        effects_positive_loop_limit_network_fetch_prepare_26_alpha_lane = effects_positive_loop_limit_network_fetch_prepare_26_alpha_lane + 3;
    } else {
        effects_positive_loop_limit_network_fetch_prepare_26_beta_lane = effects_positive_loop_limit_network_fetch_prepare_26_beta_lane + 5;
    }

    return ledger_9 + pivot_9;
}

flow effects_positive_loop_limit_network_fetch_route_26(seed: i32) -> i32 {
    var ledger_7 = seed + 27;
    var pivot_7 = ledger_7 / 9;
    var window_7 = 9;
    while window_7 > 0 limit Iterations(10) {
        pivot_7 = pivot_7 + window_7;
        ledger_7 = ledger_7 + pivot_7;
        window_7 = window_7 - 1;
    }
    if ledger_7 != pivot_7 {
        ledger_7 = ledger_7 + 9;
    }
    if seed > 9 {
        ledger_7 = ledger_7 + effects_positive_loop_limit_network_fetch_score_26(seed - 9);
    }
    var effects_positive_loop_limit_network_fetch_route_26_north_gate = seed + 27;
    var effects_positive_loop_limit_network_fetch_route_26_south_gate = effects_positive_loop_limit_network_fetch_route_26_north_gate % 10;
    var effects_positive_loop_limit_network_fetch_route_26_east_gate = effects_positive_loop_limit_network_fetch_route_26_south_gate + effects_positive_loop_limit_network_fetch_route_26_north_gate;
    while effects_positive_loop_limit_network_fetch_route_26_south_gate < 4 limit Iterations(4) {
        effects_positive_loop_limit_network_fetch_route_26_east_gate = effects_positive_loop_limit_network_fetch_route_26_east_gate + effects_positive_loop_limit_network_fetch_route_26_south_gate;
        effects_positive_loop_limit_network_fetch_route_26_south_gate = effects_positive_loop_limit_network_fetch_route_26_south_gate + 1;
    }
    if effects_positive_loop_limit_network_fetch_route_26_east_gate != effects_positive_loop_limit_network_fetch_route_26_north_gate {
        effects_positive_loop_limit_network_fetch_route_26_north_gate = effects_positive_loop_limit_network_fetch_route_26_north_gate + effects_positive_loop_limit_network_fetch_route_26_east_gate;
    }

    return ledger_7 + pivot_7;
}

flow effects_positive_loop_limit_network_fetch_score_26(seed: i32) -> i32 {
    var ledger_5 = seed + 7;
    var pivot_5 = ledger_5 * 8;
    var window_5 = 0;
    while window_5 < 10 limit Iterations(10) {
        pivot_5 = pivot_5 + window_5 + 7;
        window_5 = window_5 + 1;
    }
    if pivot_5 > 49 {
        ledger_5 = ledger_5 + pivot_5;
    } else {
        ledger_5 = ledger_5 - 7;
    }
    if seed > 7 {
        ledger_5 = ledger_5 + effects_positive_loop_limit_network_fetch_finish_26(seed - 7);
    }
    var effects_positive_loop_limit_network_fetch_score_26_red_score = seed * 10;
    var effects_positive_loop_limit_network_fetch_score_26_blue_score = effects_positive_loop_limit_network_fetch_score_26_red_score / 3;
    var effects_positive_loop_limit_network_fetch_score_26_green_score = effects_positive_loop_limit_network_fetch_score_26_blue_score + 30;
    if effects_positive_loop_limit_network_fetch_score_26_red_score >= effects_positive_loop_limit_network_fetch_score_26_green_score {
        effects_positive_loop_limit_network_fetch_score_26_green_score = effects_positive_loop_limit_network_fetch_score_26_green_score + effects_positive_loop_limit_network_fetch_score_26_red_score;
    }
    if effects_positive_loop_limit_network_fetch_score_26_blue_score <= effects_positive_loop_limit_network_fetch_score_26_green_score {
        effects_positive_loop_limit_network_fetch_score_26_blue_score = effects_positive_loop_limit_network_fetch_score_26_blue_score + 1;
    }
    effects_positive_loop_limit_network_fetch_score_26_red_score = effects_positive_loop_limit_network_fetch_score_26_red_score + effects_positive_loop_limit_network_fetch_score_26_blue_score + effects_positive_loop_limit_network_fetch_score_26_green_score;

    return ledger_5 + pivot_5;
}

flow effects_positive_loop_limit_network_fetch_finish_26(seed: i32) -> i32 {
    var ledger_18 = seed - 20;
    var pivot_18 = seed + 40;
    var window_18 = 1;
    while window_18 <= 22 limit Iterations(22) {
        ledger_18 = ledger_18 + (pivot_18 % (25));
        pivot_18 = pivot_18 + window_18;
        window_18 = window_18 + 1;
    }
    if ledger_18 < pivot_18 {
        ledger_18 = ledger_18 + pivot_18 - 20;
    }
    if seed > 20 {
        ledger_18 = ledger_18 + effects_positive_loop_limit_network_fetch_prepare_26(seed - 20);
    }
    var effects_positive_loop_limit_network_fetch_finish_26_final_seed = seed + 35;
    var effects_positive_loop_limit_network_fetch_finish_26_final_mask = effects_positive_loop_limit_network_fetch_finish_26_final_seed - 3;
    var effects_positive_loop_limit_network_fetch_finish_26_final_roll = effects_positive_loop_limit_network_fetch_finish_26_final_mask * 5;
    while effects_positive_loop_limit_network_fetch_finish_26_final_mask > 0 limit Iterations(12) {
        effects_positive_loop_limit_network_fetch_finish_26_final_roll = effects_positive_loop_limit_network_fetch_finish_26_final_roll + effects_positive_loop_limit_network_fetch_finish_26_final_mask;
        effects_positive_loop_limit_network_fetch_finish_26_final_mask = effects_positive_loop_limit_network_fetch_finish_26_final_mask - 1;
    }
    if effects_positive_loop_limit_network_fetch_finish_26_final_roll > effects_positive_loop_limit_network_fetch_finish_26_final_seed {
        effects_positive_loop_limit_network_fetch_finish_26_final_seed = effects_positive_loop_limit_network_fetch_finish_26_final_roll - effects_positive_loop_limit_network_fetch_finish_26_final_seed;
    }

    return ledger_18 + pivot_18;
}

flow main(args: Array<string>) -> i32 {
    var seed = 29;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = effects_positive_loop_limit_network_fetch_exercise_26(seed);
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
