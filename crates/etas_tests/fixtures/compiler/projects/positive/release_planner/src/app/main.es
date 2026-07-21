module app.main;

import std.collections.Array;
import app.release.plan.{build_release};
import app.release.risk.{score_release};

flow main(args: Array<string>) -> i32 ![]
{
    let release = build_release("v1.2.0");
    let level = score_release(release).level;
    return 0;
}
