module tests.compiler.policies.negative.handler_cannot_hide_denied_write;

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

spec FixturePolicy: trace = -Workspace.write & +Log.write;

flow policies_negative_handler_cannot_hide_denied_write_exercise_37(seed: i32) -> i32
    ~ FixturePolicy
{
    var total = policies_negative_handler_cannot_hide_denied_write_prepare_37(seed);
    total = total + policies_negative_handler_cannot_hide_denied_write_route_37(seed + 2);
    if total < 0 {
        total = 0 - total;
    }
    let policies_negative_handler_cannot_hide_denied_write_adjust_37: i32 -> i32 = (value: i32) => value + 37;
    total = policies_negative_handler_cannot_hide_denied_write_adjust_37(total);
    handle {
        perform Workspace.write("reports/**", "reports/out.md", "body");
    } with {
        Workspace.write(pattern, path, body) => {
            perform Log.write("would write " + path);
            resume;
        }
    };
    total = total + policies_negative_handler_cannot_hide_denied_write_score_37(2);
    total = total + policies_negative_handler_cannot_hide_denied_write_finish_37(1);
    if total > 4000 {
        total = total - 4000;
    } else {
        total = total + 68;
    }
    var policies_negative_handler_cannot_hide_denied_write_exercise_37_lambda_probe = seed + 50;
    var policies_negative_handler_cannot_hide_denied_write_exercise_37_lambda_shadow = policies_negative_handler_cannot_hide_denied_write_exercise_37_lambda_probe * 3;
    var policies_negative_handler_cannot_hide_denied_write_exercise_37_lambda_offset = policies_negative_handler_cannot_hide_denied_write_exercise_37_lambda_shadow - total;
    if policies_negative_handler_cannot_hide_denied_write_exercise_37_lambda_offset > 0 {
        total = total + policies_negative_handler_cannot_hide_denied_write_exercise_37_lambda_offset;
    } else {
        total = total - policies_negative_handler_cannot_hide_denied_write_exercise_37_lambda_offset;
    }
    policies_negative_handler_cannot_hide_denied_write_exercise_37_lambda_probe = policies_negative_handler_cannot_hide_denied_write_exercise_37_lambda_probe + total;
    policies_negative_handler_cannot_hide_denied_write_exercise_37_lambda_shadow = policies_negative_handler_cannot_hide_denied_write_exercise_37_lambda_shadow + policies_negative_handler_cannot_hide_denied_write_exercise_37_lambda_probe;

    return total;
}

flow policies_negative_handler_cannot_hide_denied_write_prepare_37(seed: i32) -> i32 {
    var ledger_2 = seed - 4;
    var pivot_2 = seed + 8;
    var window_2 = 1;
    while window_2 <= 6 limit Iterations(6) {
        ledger_2 = ledger_2 + (pivot_2 % (9));
        pivot_2 = pivot_2 + window_2;
        window_2 = window_2 + 1;
    }
    if ledger_2 < pivot_2 {
        ledger_2 = ledger_2 + pivot_2 - 4;
    }
    if seed > 4 {
        ledger_2 = ledger_2 + policies_negative_handler_cannot_hide_denied_write_route_37(seed - 4);
    }
    var policies_negative_handler_cannot_hide_denied_write_prepare_37_alpha_lane = seed + 37;
    var policies_negative_handler_cannot_hide_denied_write_prepare_37_beta_lane = policies_negative_handler_cannot_hide_denied_write_prepare_37_alpha_lane * 2;
    var policies_negative_handler_cannot_hide_denied_write_prepare_37_gamma_lane = policies_negative_handler_cannot_hide_denied_write_prepare_37_beta_lane - seed;
    policies_negative_handler_cannot_hide_denied_write_prepare_37_alpha_lane = policies_negative_handler_cannot_hide_denied_write_prepare_37_alpha_lane + policies_negative_handler_cannot_hide_denied_write_prepare_37_gamma_lane;
    policies_negative_handler_cannot_hide_denied_write_prepare_37_beta_lane = policies_negative_handler_cannot_hide_denied_write_prepare_37_beta_lane + policies_negative_handler_cannot_hide_denied_write_prepare_37_alpha_lane;
    policies_negative_handler_cannot_hide_denied_write_prepare_37_gamma_lane = policies_negative_handler_cannot_hide_denied_write_prepare_37_gamma_lane + policies_negative_handler_cannot_hide_denied_write_prepare_37_beta_lane;
    if policies_negative_handler_cannot_hide_denied_write_prepare_37_gamma_lane > policies_negative_handler_cannot_hide_denied_write_prepare_37_alpha_lane {
        policies_negative_handler_cannot_hide_denied_write_prepare_37_alpha_lane = policies_negative_handler_cannot_hide_denied_write_prepare_37_alpha_lane + 3;
    } else {
        policies_negative_handler_cannot_hide_denied_write_prepare_37_beta_lane = policies_negative_handler_cannot_hide_denied_write_prepare_37_beta_lane + 5;
    }

    return ledger_2 + pivot_2;
}

flow policies_negative_handler_cannot_hide_denied_write_route_37(seed: i32) -> i32 {
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
        ledger_7 = ledger_7 + policies_negative_handler_cannot_hide_denied_write_score_37(seed - 9);
    }
    var policies_negative_handler_cannot_hide_denied_write_route_37_north_gate = seed + 38;
    var policies_negative_handler_cannot_hide_denied_write_route_37_south_gate = policies_negative_handler_cannot_hide_denied_write_route_37_north_gate % 7;
    var policies_negative_handler_cannot_hide_denied_write_route_37_east_gate = policies_negative_handler_cannot_hide_denied_write_route_37_south_gate + policies_negative_handler_cannot_hide_denied_write_route_37_north_gate;
    while policies_negative_handler_cannot_hide_denied_write_route_37_south_gate < 4 limit Iterations(4) {
        policies_negative_handler_cannot_hide_denied_write_route_37_east_gate = policies_negative_handler_cannot_hide_denied_write_route_37_east_gate + policies_negative_handler_cannot_hide_denied_write_route_37_south_gate;
        policies_negative_handler_cannot_hide_denied_write_route_37_south_gate = policies_negative_handler_cannot_hide_denied_write_route_37_south_gate + 1;
    }
    if policies_negative_handler_cannot_hide_denied_write_route_37_east_gate != policies_negative_handler_cannot_hide_denied_write_route_37_north_gate {
        policies_negative_handler_cannot_hide_denied_write_route_37_north_gate = policies_negative_handler_cannot_hide_denied_write_route_37_north_gate + policies_negative_handler_cannot_hide_denied_write_route_37_east_gate;
    }

    return ledger_7 + pivot_7;
}

flow policies_negative_handler_cannot_hide_denied_write_score_37(seed: i32) -> i32 {
    var ledger_16 = seed + 54;
    var pivot_16 = ledger_16 / 18;
    var window_16 = 18;
    while window_16 > 0 limit Iterations(19) {
        pivot_16 = pivot_16 + window_16;
        ledger_16 = ledger_16 + pivot_16;
        window_16 = window_16 - 1;
    }
    if ledger_16 != pivot_16 {
        ledger_16 = ledger_16 + 18;
    }
    if seed > 18 {
        ledger_16 = ledger_16 + policies_negative_handler_cannot_hide_denied_write_finish_37(seed - 18);
    }
    var policies_negative_handler_cannot_hide_denied_write_score_37_red_score = seed * 3;
    var policies_negative_handler_cannot_hide_denied_write_score_37_blue_score = policies_negative_handler_cannot_hide_denied_write_score_37_red_score / 4;
    var policies_negative_handler_cannot_hide_denied_write_score_37_green_score = policies_negative_handler_cannot_hide_denied_write_score_37_blue_score + 41;
    if policies_negative_handler_cannot_hide_denied_write_score_37_red_score >= policies_negative_handler_cannot_hide_denied_write_score_37_green_score {
        policies_negative_handler_cannot_hide_denied_write_score_37_green_score = policies_negative_handler_cannot_hide_denied_write_score_37_green_score + policies_negative_handler_cannot_hide_denied_write_score_37_red_score;
    }
    if policies_negative_handler_cannot_hide_denied_write_score_37_blue_score <= policies_negative_handler_cannot_hide_denied_write_score_37_green_score {
        policies_negative_handler_cannot_hide_denied_write_score_37_blue_score = policies_negative_handler_cannot_hide_denied_write_score_37_blue_score + 1;
    }
    policies_negative_handler_cannot_hide_denied_write_score_37_red_score = policies_negative_handler_cannot_hide_denied_write_score_37_red_score + policies_negative_handler_cannot_hide_denied_write_score_37_blue_score + policies_negative_handler_cannot_hide_denied_write_score_37_green_score;

    return ledger_16 + pivot_16;
}

flow policies_negative_handler_cannot_hide_denied_write_finish_37(seed: i32) -> i32 {
    var ledger_14 = seed + 16;
    var pivot_14 = ledger_14 * 17;
    var window_14 = 0;
    while window_14 < 19 limit Iterations(19) {
        pivot_14 = pivot_14 + window_14 + 16;
        window_14 = window_14 + 1;
    }
    if pivot_14 > 112 {
        ledger_14 = ledger_14 + pivot_14;
    } else {
        ledger_14 = ledger_14 - 16;
    }
    if seed > 16 {
        ledger_14 = ledger_14 + policies_negative_handler_cannot_hide_denied_write_prepare_37(seed - 16);
    }
    var policies_negative_handler_cannot_hide_denied_write_finish_37_final_seed = seed + 46;
    var policies_negative_handler_cannot_hide_denied_write_finish_37_final_mask = policies_negative_handler_cannot_hide_denied_write_finish_37_final_seed - 2;
    var policies_negative_handler_cannot_hide_denied_write_finish_37_final_roll = policies_negative_handler_cannot_hide_denied_write_finish_37_final_mask * 4;
    while policies_negative_handler_cannot_hide_denied_write_finish_37_final_mask > 0 limit Iterations(12) {
        policies_negative_handler_cannot_hide_denied_write_finish_37_final_roll = policies_negative_handler_cannot_hide_denied_write_finish_37_final_roll + policies_negative_handler_cannot_hide_denied_write_finish_37_final_mask;
        policies_negative_handler_cannot_hide_denied_write_finish_37_final_mask = policies_negative_handler_cannot_hide_denied_write_finish_37_final_mask - 1;
    }
    if policies_negative_handler_cannot_hide_denied_write_finish_37_final_roll > policies_negative_handler_cannot_hide_denied_write_finish_37_final_seed {
        policies_negative_handler_cannot_hide_denied_write_finish_37_final_seed = policies_negative_handler_cannot_hide_denied_write_finish_37_final_roll - policies_negative_handler_cannot_hide_denied_write_finish_37_final_seed;
    }

    return ledger_14 + pivot_14;
}

flow main(args: Array<string>) -> i32 {
    var seed = 40;
    if args.len() > 0 {
        seed = seed + 1;
    } else {
        seed = seed + 2;
    }
    var result = policies_negative_handler_cannot_hide_denied_write_exercise_37(seed);
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
