module tests.compiler.effects.positive.interprocedural_fixpoint_089;


flow interprocedural_fixpoint_oasis_oasis_entry(seed: i32) -> i32 ![]
{
    var oasis_total = interprocedural_fixpoint_oasis_oasis_prepare(seed);
    oasis_total = oasis_total + interprocedural_fixpoint_oasis_oasis_route(seed + 9);
    var recursive_marker = seed - 1;
    while recursive_marker < seed limit Iterations(2) {
        recursive_marker = recursive_marker + 1;
    }
    let oasis_adjust: i32 -> i32 = (value: i32) => value + 12;
    oasis_total = oasis_adjust(oasis_total);
    oasis_total = oasis_total + interprocedural_fixpoint_oasis_oasis_score(6);
    oasis_total = oasis_total + interprocedural_fixpoint_oasis_oasis_finish(8);
    if oasis_total > 129 {
        oasis_total = oasis_total - 3;
    } else {
        oasis_total = oasis_total + 8;
    }
    return oasis_total;
}

flow interprocedural_fixpoint_oasis_oasis_prepare(seed: i32) -> i32 ![]
{
    var golf_prepare_total = seed + 16;
    var golf_prepare_cursor = 0;
    while golf_prepare_cursor < 12 limit Iterations(12) {
        golf_prepare_total = golf_prepare_total + golf_prepare_cursor + 5;
        golf_prepare_cursor = golf_prepare_cursor + 1;
    }
    if golf_prepare_total % 2 == 0 {
        golf_prepare_total = golf_prepare_total + interprocedural_fixpoint_oasis_oasis_score(1);
    } else {
        golf_prepare_total = golf_prepare_total - 5;
    }
    var golf_prepare_left = golf_prepare_total + seed;
    var golf_prepare_right = golf_prepare_left * 3;
    var golf_prepare_merged = golf_prepare_right - golf_prepare_left;
    if golf_prepare_merged > 27 {
        golf_prepare_total = golf_prepare_total + golf_prepare_merged;
    }
    return golf_prepare_total;
}

flow interprocedural_fixpoint_oasis_oasis_route(seed: i32) -> i32 ![]
{
    var golf_route_total = seed * 16;
    var golf_route_cursor = 0;
    while golf_route_cursor < 12 limit Iterations(12) {
        golf_route_total = golf_route_total + golf_route_cursor + 5;
        golf_route_cursor = golf_route_cursor + 1;
    }
    if golf_route_total % 2 == 0 {
        golf_route_total = golf_route_total + 25;
    } else {
        golf_route_total = golf_route_total - 5;
    }
    var golf_route_left = golf_route_total + seed;
    var golf_route_right = golf_route_left * 3;
    var golf_route_merged = golf_route_right - golf_route_left;
    if golf_route_merged > 27 {
        golf_route_total = golf_route_total + golf_route_merged;
    }
    return golf_route_total;
}

flow interprocedural_fixpoint_oasis_oasis_score(seed: i32) -> i32 ![]
{
    var golf_score_total = seed + 16;
    var golf_score_cursor = 0;
    while golf_score_cursor < 11 limit Iterations(11) {
        golf_score_total = golf_score_total + golf_score_cursor + 5;
        golf_score_cursor = golf_score_cursor + 1;
    }
    if golf_score_total % 2 == 0 {
        golf_score_total = golf_score_total + 25;
    } else {
        golf_score_total = golf_score_total - 5;
    }
    var golf_score_left = golf_score_total + seed;
    var golf_score_right = golf_score_left * 3;
    var golf_score_merged = golf_score_right - golf_score_left;
    if golf_score_merged > 27 {
        golf_score_total = golf_score_total + golf_score_merged;
    }
    return golf_score_total;
}

flow interprocedural_fixpoint_oasis_oasis_finish(seed: i32) -> i32 ![]
{
    var golf_finish_total = seed - 16;
    var golf_finish_cursor = 0;
    while golf_finish_cursor < 6 limit Iterations(6) {
        golf_finish_total = golf_finish_total + golf_finish_cursor + 5;
        golf_finish_cursor = golf_finish_cursor + 1;
    }
    if golf_finish_total % 2 == 0 {
        golf_finish_total = golf_finish_total + 25;
    } else {
        golf_finish_total = golf_finish_total - 5;
    }
    var golf_finish_left = golf_finish_total + seed;
    var golf_finish_right = golf_finish_left * 3;
    var golf_finish_merged = golf_finish_right - golf_finish_left;
    if golf_finish_merged > 27 {
        golf_finish_total = golf_finish_total + golf_finish_merged;
    }
    return golf_finish_total;
}

flow main(args: Array<string>) -> i32 ![]
{
    var oasis_seed = 2;
    if args.len() > 0 {
        oasis_seed = oasis_seed + 1;
    } else {
        oasis_seed = oasis_seed + 2;
    }
    let oasis_result = interprocedural_fixpoint_oasis_oasis_entry(oasis_seed);
    if oasis_result > 0 {
        return 0;
    }
    return 1;
}
